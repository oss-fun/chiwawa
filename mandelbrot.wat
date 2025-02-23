(module $mandelbrot-8c1d6e9a665ef7b9.wasm
  (type (;0;) (func (param i32)))
  (type (;1;) (func))
  (type (;2;) (func (param i32 i32)))
  (type (;3;) (func (param i32 i32 i32)))
  (type (;4;) (func (param i32 i32) (result i32)))
  (type (;5;) (func (param i32) (result i32)))
  (type (;6;) (func (param i32 i32 i32) (result i32)))
  (type (;7;) (func (param i32 i32 i32 i32 i32)))
  (type (;8;) (func (param i32 i32 i32 i32 i32 i32)))
  (type (;9;) (func (param i32 f64 i32) (result i32)))
  (type (;10;) (func (param i32 i32 i32 i32) (result i32)))
  (type (;11;) (func (result i32)))
  (type (;12;) (func (param i32 i32 i32 i32)))
  (type (;13;) (func (param i32 i32 i32 i32 i32) (result i32)))
  (type (;14;) (func (result i64)))
  (type (;15;) (func (param i32 i64 i32)))
  (type (;16;) (func (param i32 i64)))
  (type (;17;) (func (param i32 i32 i32 i32 i32 i32) (result i32)))
  (type (;18;) (func (param i32 i32 i32 i32 i32 i32 i32)))
  (type (;19;) (func (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
  (type (;20;) (func (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32)))
  (func $_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h324098578815f6c0E.llvm.12408529821376129927 (type 5) (param i32) (result i32)
    local.get 0
    i32.load
    call $_ZN3std3sys9backtrace28__rust_begin_short_backtrace17hbe4fecc903c07c99E
    i32.const 0)
  (func $_ZN3std3sys12thread_local6statik20LazyStorage$LT$T$GT$10initialize17ha6e918742de6013dE (type 4) (param i32 i32) (result i32)
    (local i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.load
        local.set 2
        local.get 1
        i32.const 0
        i32.store
        local.get 2
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.load offset=4
        local.set 2
        br 1 (;@1;)
      end
      call $_ZN15crossbeam_epoch7default17default_collector17h7fa099018adad5efE
      call $_ZN15crossbeam_epoch9collector9Collector8register17h6bed026672a5d855E
      local.set 2
    end
    local.get 0
    i32.load offset=4
    local.set 1
    local.get 0
    local.get 2
    i32.store offset=4
    local.get 0
    i32.load
    local.set 2
    local.get 0
    i32.const 1
    i32.store
    local.get 0
    i32.const 4
    i32.add
    local.set 0
    block  ;; label = @1
      local.get 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.get 1
      i32.load offset=1040
      local.tee 2
      i32.const -1
      i32.add
      i32.store offset=1040
      local.get 1
      i32.load offset=1036
      br_if 0 (;@1;)
      local.get 2
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 1
      call $_ZN15crossbeam_epoch8internal5Local8finalize17ha822a731c89268d9E
    end
    local.get 0)
  (func $_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17hb0525cf1cd8205dbE.llvm.12408529821376129927 (type 5) (param i32) (result i32)
    local.get 0
    i32.load
    call $_ZN3std3sys9backtrace28__rust_begin_short_backtrace17hbe4fecc903c07c99E
    i32.const 0)
  (func $_ZN4core9panicking13assert_failed17he9db8920dd41e943E (type 7) (param i32 i32 i32 i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 5
    global.set $__stack_pointer
    local.get 5
    local.get 2
    i32.store offset=12
    local.get 5
    local.get 1
    i32.store offset=8
    local.get 0
    local.get 5
    i32.const 8
    i32.add
    i32.const 1048856
    local.get 5
    i32.const 12
    i32.add
    i32.const 1048856
    local.get 3
    local.get 4
    call $_ZN4core9panicking19assert_failed_inner17he4920e028524a869E
    unreachable)
  (func $_ZN4core9panicking13assert_failed17hf847e9f68e665e06E (type 7) (param i32 i32 i32 i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 5
    global.set $__stack_pointer
    local.get 5
    local.get 2
    i32.store offset=12
    local.get 5
    local.get 1
    i32.store offset=8
    local.get 0
    local.get 5
    i32.const 8
    i32.add
    i32.const 1048872
    local.get 5
    i32.const 12
    i32.add
    i32.const 1048872
    local.get 3
    local.get 4
    call $_ZN4core9panicking19assert_failed_inner17he4920e028524a869E
    unreachable)
  (func $_ZN5rayon4iter8plumbing24bridge_producer_consumer6helper17h124c3a64ae04bff2E (type 8) (param i32 i32 i32 i32 i32 i32)
    (local i32 i32)
    global.get $__stack_pointer
    i32.const 80
    i32.sub
    local.tee 6
    global.set $__stack_pointer
    local.get 6
    local.get 0
    i32.store offset=4
    local.get 6
    local.get 3
    i32.store offset=12
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 0
              i32.const 1
              i32.shr_u
              local.tee 0
              local.get 3
              i32.lt_u
              br_if 0 (;@5;)
              local.get 1
              br_if 1 (;@4;)
              local.get 2
              i32.eqz
              br_if 0 (;@5;)
              local.get 2
              i32.const 1
              i32.shr_u
              local.set 3
              br 2 (;@3;)
            end
            local.get 4
            i32.load offset=8
            local.tee 0
            i32.eqz
            br_if 3 (;@1;)
            local.get 4
            i32.load
            local.set 2
            local.get 4
            i32.load offset=12
            local.tee 3
            local.set 1
            i32.const 0
            local.set 7
            block  ;; label = @5
              local.get 4
              i32.load offset=4
              local.tee 4
              i32.eqz
              br_if 0 (;@5;)
              local.get 4
              local.get 0
              i32.div_u
              local.tee 1
              local.get 4
              local.get 1
              local.get 0
              i32.mul
              i32.sub
              i32.const 0
              i32.ne
              i32.add
              local.tee 7
              local.get 3
              i32.add
              local.set 1
            end
            local.get 6
            local.get 5
            i32.store offset=76
            i32.const 0
            local.get 1
            local.get 3
            i32.sub
            local.tee 5
            local.get 5
            local.get 1
            i32.gt_u
            select
            local.tee 5
            local.get 7
            local.get 5
            local.get 7
            i32.lt_u
            select
            local.tee 5
            i32.eqz
            br_if 2 (;@2;)
            loop  ;; label = @5
              local.get 6
              local.get 4
              local.get 0
              local.get 4
              local.get 0
              i32.lt_u
              select
              i32.store offset=24
              local.get 4
              local.get 0
              i32.sub
              local.set 4
              local.get 6
              local.get 2
              i32.store offset=20
              local.get 2
              local.get 0
              i32.add
              local.set 2
              local.get 6
              local.get 3
              i32.store offset=16
              local.get 3
              i32.const 1
              i32.add
              local.set 3
              local.get 6
              i32.const 76
              i32.add
              local.get 6
              i32.const 16
              i32.add
              call $_ZN4core3ops8function5impls71_$LT$impl$u20$core..ops..function..FnMut$LT$A$GT$$u20$for$u20$$RF$F$GT$8call_mut17ha6b8a309530b5fabE
              local.get 5
              i32.const -1
              i32.add
              local.tee 5
              br_if 0 (;@5;)
              br 3 (;@2;)
            end
          end
          call $_ZN10rayon_core19current_num_threads17h7db6c4811dab6784E
          local.tee 3
          local.get 2
          i32.const 1
          i32.shr_u
          local.tee 2
          local.get 3
          local.get 2
          i32.gt_u
          select
          local.set 3
        end
        local.get 6
        local.get 3
        i32.store offset=8
        local.get 4
        i32.load offset=4
        local.set 3
        local.get 6
        local.get 0
        i32.store offset=76
        local.get 4
        i32.load
        local.set 1
        local.get 4
        i32.load offset=8
        local.set 2
        local.get 4
        i32.load offset=12
        local.set 4
        local.get 6
        local.get 5
        i32.store offset=72
        local.get 6
        local.get 4
        i32.store offset=68
        local.get 6
        local.get 2
        i32.store offset=64
        local.get 6
        local.get 1
        i32.store offset=56
        local.get 6
        local.get 5
        i32.store offset=44
        local.get 6
        local.get 4
        local.get 0
        i32.add
        i32.store offset=40
        local.get 6
        local.get 2
        i32.store offset=36
        local.get 6
        local.get 2
        local.get 0
        i32.mul
        local.tee 0
        local.get 3
        local.get 0
        local.get 3
        i32.lt_u
        select
        local.tee 0
        i32.store offset=60
        local.get 6
        local.get 3
        local.get 0
        i32.sub
        i32.store offset=32
        local.get 6
        local.get 1
        local.get 0
        i32.add
        i32.store offset=28
        local.get 6
        local.get 6
        i32.const 8
        i32.add
        i32.store offset=52
        local.get 6
        local.get 6
        i32.const 76
        i32.add
        i32.store offset=48
        local.get 6
        local.get 6
        i32.const 8
        i32.add
        i32.store offset=24
        local.get 6
        local.get 6
        i32.const 76
        i32.add
        i32.store offset=20
        local.get 6
        local.get 6
        i32.const 4
        i32.add
        i32.store offset=16
        block  ;; label = @3
          i32.const 0
          i32.load offset=1058996
          local.tee 0
          br_if 0 (;@3;)
          call $_ZN10rayon_core8registry15global_registry17hf6a3fbcd34bc87c8E
          i32.load
          local.set 4
          block  ;; label = @4
            i32.const 0
            i32.load offset=1058996
            local.tee 0
            br_if 0 (;@4;)
            local.get 4
            i32.const 64
            i32.add
            local.get 6
            i32.const 16
            i32.add
            call $_ZN10rayon_core8registry8Registry14in_worker_cold17h3a0b00e35b93607aE.llvm.2793606751137166678
            br 2 (;@2;)
          end
          block  ;; label = @4
            local.get 0
            i32.load offset=140
            local.get 4
            i32.ne
            br_if 0 (;@4;)
            local.get 6
            i32.const 16
            i32.add
            local.get 0
            call $_ZN10rayon_core4join12join_context28_$u7b$$u7b$closure$u7d$$u7d$17h8fe34789bad32535E.llvm.2793606751137166678
            br 2 (;@2;)
          end
          local.get 4
          i32.const 64
          i32.add
          local.get 0
          local.get 6
          i32.const 16
          i32.add
          call $_ZN10rayon_core8registry8Registry15in_worker_cross17h4ec1b8df6cedc729E.llvm.2793606751137166678
          br 1 (;@2;)
        end
        local.get 6
        i32.const 16
        i32.add
        local.get 0
        call $_ZN10rayon_core4join12join_context28_$u7b$$u7b$closure$u7d$$u7d$17h8fe34789bad32535E.llvm.2793606751137166678
      end
      local.get 6
      i32.const 80
      i32.add
      global.set $__stack_pointer
      return
    end
    local.get 6
    i32.const 0
    i32.store offset=32
    local.get 6
    i32.const 1
    i32.store offset=20
    local.get 6
    i32.const 1048704
    i32.store offset=16
    local.get 6
    i64.const 4
    i64.store offset=24 align=4
    local.get 6
    i32.const 16
    i32.add
    i32.const 1048712
    call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
    unreachable)
  (func $_ZN98_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter..SpecFromIter$LT$T$C$I$GT$$GT$9from_iter17h3a1156df5a7789eeE (type 3) (param i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 f64)
    i32.const 0
    local.set 3
    i32.const 0
    local.get 1
    i32.load offset=8
    local.tee 4
    local.get 1
    i32.load offset=4
    local.tee 5
    i32.sub
    local.tee 6
    local.get 6
    local.get 4
    i32.gt_u
    select
    local.tee 7
    i32.const 6
    i32.shl
    local.set 8
    block  ;; label = @1
      local.get 7
      i32.const 67108863
      i32.gt_u
      br_if 0 (;@1;)
      local.get 8
      i32.const 2147483616
      i32.gt_u
      br_if 0 (;@1;)
      i32.const 0
      local.set 9
      block  ;; label = @2
        block  ;; label = @3
          local.get 8
          br_if 0 (;@3;)
          i32.const 32
          local.set 10
          i32.const 0
          local.set 7
          br 1 (;@2;)
        end
        i32.const 0
        i32.load8_u offset=1058985
        drop
        i32.const 32
        local.set 3
        local.get 8
        i32.const 32
        call $__rust_alloc
        local.tee 10
        i32.eqz
        br_if 1 (;@1;)
      end
      block  ;; label = @2
        local.get 4
        local.get 5
        i32.le_u
        br_if 0 (;@2;)
        local.get 1
        i32.load
        local.set 4
        local.get 5
        i32.const 3
        i32.shl
        local.set 8
        i32.const 0
        local.set 9
        local.get 10
        local.set 1
        loop  ;; label = @3
          local.get 1
          i32.const 56
          i32.add
          local.get 4
          f64.load
          local.tee 11
          local.get 8
          f64.convert_i32_u
          f64.mul
          f64.const -0x1.8p+0 (;=-1.5;)
          f64.add
          f64.store
          local.get 1
          i32.const 48
          i32.add
          local.get 11
          local.get 8
          i32.const 1
          i32.add
          f64.convert_i32_u
          f64.mul
          f64.const -0x1.8p+0 (;=-1.5;)
          f64.add
          f64.store
          local.get 1
          i32.const 40
          i32.add
          local.get 11
          local.get 8
          i32.const 2
          i32.add
          f64.convert_i32_u
          f64.mul
          f64.const -0x1.8p+0 (;=-1.5;)
          f64.add
          f64.store
          local.get 1
          i32.const 32
          i32.add
          local.get 11
          local.get 8
          i32.const 3
          i32.add
          f64.convert_i32_u
          f64.mul
          f64.const -0x1.8p+0 (;=-1.5;)
          f64.add
          f64.store
          local.get 1
          i32.const 24
          i32.add
          local.get 11
          local.get 8
          i32.const 4
          i32.add
          f64.convert_i32_u
          f64.mul
          f64.const -0x1.8p+0 (;=-1.5;)
          f64.add
          f64.store
          local.get 1
          i32.const 16
          i32.add
          local.get 11
          local.get 8
          i32.const 5
          i32.add
          f64.convert_i32_u
          f64.mul
          f64.const -0x1.8p+0 (;=-1.5;)
          f64.add
          f64.store
          local.get 1
          i32.const 8
          i32.add
          local.get 11
          local.get 8
          i32.const 6
          i32.add
          f64.convert_i32_u
          f64.mul
          f64.const -0x1.8p+0 (;=-1.5;)
          f64.add
          f64.store
          local.get 1
          local.get 11
          local.get 8
          i32.const 7
          i32.add
          f64.convert_i32_u
          f64.mul
          f64.const -0x1.8p+0 (;=-1.5;)
          f64.add
          f64.store
          local.get 8
          i32.const 8
          i32.add
          local.set 8
          local.get 1
          i32.const 64
          i32.add
          local.set 1
          local.get 6
          local.get 9
          i32.const 1
          i32.add
          local.tee 9
          i32.ne
          br_if 0 (;@3;)
        end
      end
      local.get 0
      local.get 9
      i32.store offset=8
      local.get 0
      local.get 10
      i32.store offset=4
      local.get 0
      local.get 7
      i32.store
      return
    end
    local.get 3
    local.get 8
    local.get 2
    call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
    unreachable)
  (func $_ZN10rayon_core4join12join_context28_$u7b$$u7b$closure$u7d$$u7d$17h8fe34789bad32535E (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 96
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 16
    i32.add
    i32.const 8
    i32.add
    local.get 0
    i32.const 8
    i32.add
    i64.load align=4
    i64.store
    local.get 2
    i32.const 16
    i32.add
    i32.const 16
    i32.add
    local.get 0
    i32.const 16
    i32.add
    i64.load align=4
    i64.store
    local.get 2
    i32.const 16
    i32.add
    i32.const 24
    i32.add
    local.get 0
    i32.const 24
    i32.add
    i64.load align=4
    i64.store
    local.get 2
    i32.const 0
    i32.store8 offset=72
    local.get 2
    i32.const 0
    i32.store offset=64
    local.get 2
    i32.const 0
    i32.store offset=48
    local.get 2
    local.get 1
    i32.load offset=136
    i32.store offset=68
    local.get 2
    local.get 1
    i32.const 140
    i32.add
    i32.store offset=60
    local.get 2
    local.get 0
    i64.load align=4
    i64.store offset=16
    local.get 1
    i32.const 144
    i32.add
    local.set 3
    block  ;; label = @1
      local.get 1
      i32.load offset=144
      local.tee 4
      i32.load offset=132
      local.tee 5
      local.get 4
      i32.load offset=128
      i32.sub
      local.tee 6
      local.get 1
      i32.load offset=152
      local.tee 4
      i32.lt_s
      br_if 0 (;@1;)
      local.get 3
      local.get 4
      i32.const 1
      i32.shl
      call $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$6resize17h3d4716eead8d3956E.llvm.6352068949975026865
      local.get 1
      i32.load offset=152
      local.set 4
    end
    local.get 1
    i32.load offset=148
    local.get 4
    i32.const -1
    i32.add
    local.get 5
    i32.and
    i32.const 3
    i32.shl
    i32.add
    local.tee 4
    i32.const 5
    i32.store
    local.get 4
    local.get 2
    i32.const 16
    i32.add
    i32.store offset=4
    local.get 1
    i32.load offset=144
    local.get 5
    i32.const 1
    i32.add
    i32.store offset=132
    local.get 1
    i32.load offset=140
    local.set 4
    block  ;; label = @1
      loop  ;; label = @2
        block  ;; label = @3
          local.get 4
          i32.load offset=248
          local.tee 5
          i32.const 65536
          i32.and
          i32.eqz
          br_if 0 (;@3;)
          local.get 5
          local.set 7
          br 2 (;@1;)
        end
        local.get 4
        local.get 5
        i32.const 65536
        i32.or
        local.tee 7
        local.get 4
        i32.load offset=248
        local.tee 8
        local.get 8
        local.get 5
        i32.eq
        select
        i32.store offset=248
        local.get 8
        local.get 5
        i32.ne
        br_if 0 (;@2;)
      end
    end
    block  ;; label = @1
      local.get 7
      i32.const 255
      i32.and
      local.tee 5
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 6
        i32.const 0
        i32.gt_s
        br_if 0 (;@2;)
        local.get 7
        i32.const 8
        i32.shr_u
        i32.const 255
        i32.and
        local.get 5
        i32.ne
        br_if 1 (;@1;)
      end
      local.get 4
      i32.const 236
      i32.add
      i32.const 1
      call $_ZN10rayon_core5sleep5Sleep16wake_any_threads17h17244bbca7ed393dE
    end
    local.get 0
    i32.load offset=36
    local.set 5
    local.get 0
    i32.load offset=32
    local.set 4
    local.get 2
    i32.const 88
    i32.add
    local.get 0
    i32.const 48
    i32.add
    i64.load align=4
    i64.store
    local.get 2
    local.get 0
    i64.load offset=40 align=4
    i64.store offset=80
    local.get 4
    i32.load
    i32.const 1
    local.get 5
    i32.load
    local.get 5
    i32.load offset=4
    local.get 2
    i32.const 80
    i32.add
    local.get 0
    i32.load offset=56
    call $_ZN5rayon4iter8plumbing24bridge_producer_consumer6helper17h124c3a64ae04bff2E
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        i32.load offset=64
        i32.const 3
        i32.eq
        br_if 0 (;@2;)
        local.get 1
        i32.const 160
        i32.add
        local.set 4
        block  ;; label = @3
          block  ;; label = @4
            loop  ;; label = @5
              local.get 2
              i32.const 8
              i32.add
              local.get 3
              call $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$3pop17h1ff9952805e7f2a6E
              block  ;; label = @6
                block  ;; label = @7
                  local.get 2
                  i32.load offset=8
                  local.tee 5
                  i32.eqz
                  br_if 0 (;@7;)
                  local.get 2
                  i32.load offset=12
                  local.set 8
                  br 1 (;@6;)
                end
                loop  ;; label = @7
                  local.get 2
                  i32.const 80
                  i32.add
                  local.get 4
                  call $_ZN15crossbeam_deque5deque16Stealer$LT$T$GT$5steal17h92560776915b651eE
                  local.get 2
                  i32.load offset=80
                  local.tee 5
                  i32.const 2
                  i32.eq
                  br_if 0 (;@7;)
                end
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 5
                    br_table 0 (;@8;) 1 (;@7;) 0 (;@8;)
                  end
                  local.get 2
                  i32.load offset=64
                  i32.const 3
                  i32.ne
                  br_if 3 (;@4;)
                  br 5 (;@2;)
                end
                local.get 2
                i32.load offset=88
                local.set 8
                local.get 2
                i32.load offset=84
                local.set 5
              end
              block  ;; label = @6
                block  ;; label = @7
                  local.get 5
                  i32.const 5
                  i32.ne
                  br_if 0 (;@7;)
                  local.get 2
                  i32.const 16
                  i32.add
                  local.get 8
                  i32.eq
                  br_if 1 (;@6;)
                end
                local.get 8
                local.get 5
                call_indirect (type 0)
                local.get 2
                i32.load offset=64
                i32.const 3
                i32.ne
                br_if 1 (;@5;)
                br 4 (;@2;)
              end
            end
            local.get 2
            i32.load offset=16
            local.tee 8
            i32.eqz
            br_if 1 (;@3;)
            local.get 2
            i32.load offset=56
            local.set 4
            local.get 2
            i32.load offset=52
            local.set 3
            local.get 2
            i32.load offset=48
            local.set 7
            local.get 2
            i32.load offset=44
            local.set 0
            local.get 2
            i32.load offset=24
            local.set 5
            local.get 2
            i32.load offset=20
            local.set 1
            local.get 2
            i32.const 88
            i32.add
            local.get 2
            i32.const 36
            i32.add
            i64.load align=4
            i64.store
            local.get 2
            local.get 2
            i64.load offset=28 align=4
            i64.store offset=80
            local.get 8
            i32.load
            local.get 1
            i32.load
            i32.sub
            i32.const 1
            local.get 5
            i32.load
            local.get 5
            i32.load offset=4
            local.get 2
            i32.const 80
            i32.add
            local.get 0
            call $_ZN5rayon4iter8plumbing24bridge_producer_consumer6helper17h124c3a64ae04bff2E
            local.get 7
            i32.const 2
            i32.lt_u
            br_if 3 (;@1;)
            block  ;; label = @5
              local.get 4
              i32.load
              local.tee 5
              i32.eqz
              br_if 0 (;@5;)
              local.get 3
              local.get 5
              call_indirect (type 0)
            end
            local.get 4
            i32.load offset=4
            local.tee 5
            i32.eqz
            br_if 3 (;@1;)
            local.get 3
            local.get 5
            local.get 4
            i32.load offset=8
            call $__rust_dealloc
            br 3 (;@1;)
          end
          local.get 1
          local.get 2
          i32.const 64
          i32.add
          call $_ZN10rayon_core8registry12WorkerThread15wait_until_cold17hdc4a872cdf442250E
          br 1 (;@2;)
        end
        i32.const 1049044
        call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
        unreachable
      end
      local.get 2
      i32.load offset=48
      local.tee 5
      i32.const 1
      i32.eq
      br_if 0 (;@1;)
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 5
            br_table 1 (;@3;) 0 (;@4;) 2 (;@2;) 1 (;@3;)
          end
          unreachable
        end
        i32.const 1048888
        i32.const 40
        i32.const 1049028
        call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
        unreachable
      end
      local.get 2
      i32.load offset=52
      local.get 2
      i32.load offset=56
      call $_ZN10rayon_core6unwind16resume_unwinding17h9301afba3ac4692cE
      unreachable
    end
    local.get 2
    i32.const 96
    i32.add
    global.set $__stack_pointer)
  (func $_ZN83_$LT$rayon_core..job..StackJob$LT$L$C$F$C$R$GT$$u20$as$u20$rayon_core..job..Job$GT$7execute17h2226c5c9c3f32125E (type 0) (param i32)
    (local i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 0
    i32.load
    local.set 2
    local.get 0
    i32.const 0
    i32.store
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        i32.eqz
        br_if 0 (;@2;)
        local.get 0
        i32.load offset=28
        local.set 3
        local.get 0
        i32.load offset=8
        local.set 4
        local.get 0
        i32.load offset=4
        local.set 5
        local.get 1
        i32.const 8
        i32.add
        local.get 0
        i32.const 20
        i32.add
        i64.load align=4
        i64.store
        local.get 1
        local.get 0
        i64.load offset=12 align=4
        i64.store
        local.get 2
        i32.load
        local.get 5
        i32.load
        i32.sub
        i32.const 1
        local.get 4
        i32.load
        local.get 4
        i32.load offset=4
        local.get 1
        local.get 3
        call $_ZN5rayon4iter8plumbing24bridge_producer_consumer6helper17h124c3a64ae04bff2E
        block  ;; label = @3
          local.get 0
          i32.load offset=32
          i32.const 2
          i32.lt_u
          br_if 0 (;@3;)
          local.get 0
          i32.load offset=36
          local.set 4
          block  ;; label = @4
            local.get 0
            i32.load offset=40
            local.tee 2
            i32.load
            local.tee 3
            i32.eqz
            br_if 0 (;@4;)
            local.get 4
            local.get 3
            call_indirect (type 0)
          end
          local.get 2
          i32.load offset=4
          local.tee 3
          i32.eqz
          br_if 0 (;@3;)
          local.get 4
          local.get 3
          local.get 2
          i32.load offset=8
          call $__rust_dealloc
        end
        local.get 0
        i32.const 1
        i32.store offset=32
        local.get 0
        i32.load offset=44
        i32.load
        local.set 2
        block  ;; label = @3
          block  ;; label = @4
            local.get 0
            i32.load8_u offset=56
            br_if 0 (;@4;)
            local.get 0
            i32.load offset=48
            local.set 4
            local.get 0
            i32.const 3
            i32.store offset=48
            local.get 4
            i32.const 2
            i32.ne
            br_if 1 (;@3;)
            local.get 2
            i32.const 64
            i32.add
            local.get 0
            i32.load offset=52
            call $_ZN10rayon_core8registry8Registry26notify_worker_latch_is_set17h0c173997f1598668E
            br 1 (;@3;)
          end
          local.get 2
          local.get 2
          i32.load
          local.tee 4
          i32.const 1
          i32.add
          i32.store
          local.get 4
          i32.const 0
          i32.lt_s
          br_if 2 (;@1;)
          local.get 0
          i32.load offset=48
          local.set 4
          local.get 0
          i32.const 3
          i32.store offset=48
          local.get 1
          local.get 2
          i32.store
          block  ;; label = @4
            local.get 4
            i32.const 2
            i32.ne
            br_if 0 (;@4;)
            local.get 2
            i32.const 64
            i32.add
            local.get 0
            i32.load offset=52
            call $_ZN10rayon_core8registry8Registry26notify_worker_latch_is_set17h0c173997f1598668E
          end
          local.get 1
          i32.load
          local.tee 0
          local.get 0
          i32.load
          local.tee 0
          i32.const -1
          i32.add
          i32.store
          local.get 0
          i32.const 1
          i32.ne
          br_if 0 (;@3;)
          local.get 1
          call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h76aa42d8034d2782E
        end
        local.get 1
        i32.const 16
        i32.add
        global.set $__stack_pointer
        return
      end
      i32.const 1049248
      call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
    end
    unreachable)
  (func $_ZN4core3ops8function5impls71_$LT$impl$u20$core..ops..function..FnMut$LT$A$GT$$u20$for$u20$$RF$F$GT$8call_mut17ha6b8a309530b5fabE (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 f64)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 1
    i32.load offset=8
    local.set 3
    local.get 1
    i32.load offset=4
    local.set 4
    local.get 1
    i32.load
    local.set 5
    local.get 2
    local.get 0
    i32.load
    local.tee 0
    i32.load
    local.tee 6
    i32.load offset=8
    local.tee 1
    i32.store offset=8
    local.get 2
    local.get 0
    i32.load offset=4
    i32.load
    local.tee 7
    i32.const 3
    i32.shr_u
    local.tee 8
    i32.store offset=12
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 1
          local.get 8
          i32.ne
          br_if 0 (;@3;)
          local.get 2
          local.get 3
          i32.store offset=16
          local.get 2
          local.get 1
          i32.store offset=20
          local.get 3
          local.get 1
          i32.ne
          br_if 1 (;@2;)
          block  ;; label = @4
            local.get 7
            i32.const 8
            i32.lt_u
            br_if 0 (;@4;)
            local.get 0
            i32.load offset=8
            f64.load
            local.get 5
            f64.convert_i32_u
            f64.mul
            f64.const -0x1p+0 (;=-1;)
            f64.add
            local.set 9
            i32.const 0
            local.set 0
            i32.const 0
            local.set 8
            i32.const 0
            local.set 1
            loop  ;; label = @5
              local.get 3
              local.get 1
              i32.eq
              br_if 4 (;@1;)
              local.get 4
              local.get 1
              i32.add
              local.get 6
              i32.load offset=4
              local.get 0
              i32.add
              local.get 9
              local.get 8
              call $_ZN10mandelbrot5mand817h54a8e68c56211857E.llvm.7690246549767636906
              local.tee 8
              i32.store8
              local.get 0
              i32.const 64
              i32.add
              local.set 0
              local.get 3
              local.get 1
              i32.const 1
              i32.add
              local.tee 1
              i32.ne
              br_if 0 (;@5;)
            end
          end
          local.get 2
          i32.const 48
          i32.add
          global.set $__stack_pointer
          return
        end
        local.get 2
        i32.const 0
        i32.store offset=24
        i32.const 0
        local.get 2
        i32.const 8
        i32.add
        local.get 2
        i32.const 12
        i32.add
        local.get 2
        i32.const 24
        i32.add
        i32.const 1049400
        call $_ZN4core9panicking13assert_failed17he9db8920dd41e943E
        unreachable
      end
      local.get 2
      i32.const 0
      i32.store offset=24
      i32.const 0
      local.get 2
      i32.const 16
      i32.add
      local.get 2
      i32.const 20
      i32.add
      local.get 2
      i32.const 24
      i32.add
      i32.const 1049416
      call $_ZN4core9panicking13assert_failed17he9db8920dd41e943E
      unreachable
    end
    local.get 3
    local.get 3
    i32.const 1049432
    call $_ZN4core9panicking18panic_bounds_check17h6c9fc24fb71a0cb6E
    unreachable)
  (func $_ZN10mandelbrot5mand817h54a8e68c56211857E.llvm.7690246549767636906 (type 9) (param i32 f64 i32) (result i32)
    (local f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64 f64)
    local.get 0
    f64.load offset=56
    local.tee 3
    local.get 3
    f64.mul
    local.set 4
    local.get 0
    f64.load offset=48
    local.tee 5
    local.get 5
    f64.mul
    local.set 6
    local.get 0
    f64.load offset=40
    local.tee 7
    local.get 7
    f64.mul
    local.set 8
    local.get 0
    f64.load offset=32
    local.tee 9
    local.get 9
    f64.mul
    local.set 10
    local.get 0
    f64.load offset=24
    local.tee 11
    local.get 11
    f64.mul
    local.set 12
    local.get 0
    f64.load offset=16
    local.tee 13
    local.get 13
    f64.mul
    local.set 14
    local.get 0
    f64.load offset=8
    local.tee 15
    local.get 15
    f64.mul
    local.set 16
    local.get 0
    f64.load
    local.tee 17
    local.get 17
    f64.mul
    local.set 18
    i32.const 12
    local.set 0
    local.get 2
    i32.const 255
    i32.and
    local.set 2
    local.get 1
    local.set 19
    local.get 1
    local.set 20
    local.get 1
    local.set 21
    local.get 1
    local.set 22
    local.get 1
    local.set 23
    local.get 1
    local.set 24
    local.get 1
    local.set 25
    local.get 1
    local.set 26
    local.get 1
    local.get 1
    f64.mul
    local.tee 27
    local.set 28
    local.get 27
    local.set 29
    local.get 27
    local.set 30
    local.get 27
    local.set 31
    local.get 27
    local.set 32
    local.get 27
    local.set 33
    local.get 27
    local.set 34
    local.get 17
    local.set 35
    local.get 15
    local.set 36
    local.get 13
    local.set 37
    local.get 11
    local.set 38
    local.get 9
    local.set 39
    local.get 7
    local.set 40
    local.get 5
    local.set 41
    local.get 3
    local.set 42
    loop  ;; label = @1
      local.get 3
      local.get 3
      local.get 3
      local.get 3
      local.get 4
      local.get 27
      f64.sub
      f64.add
      local.tee 43
      local.get 43
      f64.mul
      local.get 19
      local.get 42
      local.get 42
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 19
      local.get 19
      f64.mul
      f64.sub
      f64.add
      local.tee 42
      local.get 42
      f64.mul
      local.get 43
      local.get 43
      f64.add
      local.get 19
      f64.mul
      local.get 1
      f64.add
      local.tee 19
      local.get 19
      f64.mul
      f64.sub
      f64.add
      local.tee 43
      local.get 43
      f64.mul
      local.get 19
      local.get 42
      local.get 42
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 19
      local.get 19
      f64.mul
      f64.sub
      f64.add
      local.tee 42
      local.get 42
      f64.mul
      local.set 4
      local.get 19
      local.get 43
      local.get 43
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 19
      local.get 19
      f64.mul
      local.set 27
      local.get 5
      local.get 5
      local.get 5
      local.get 5
      local.get 6
      local.get 28
      f64.sub
      f64.add
      local.tee 43
      local.get 43
      f64.mul
      local.get 20
      local.get 41
      local.get 41
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 20
      local.get 20
      f64.mul
      f64.sub
      f64.add
      local.tee 41
      local.get 41
      f64.mul
      local.get 43
      local.get 43
      f64.add
      local.get 20
      f64.mul
      local.get 1
      f64.add
      local.tee 20
      local.get 20
      f64.mul
      f64.sub
      f64.add
      local.tee 43
      local.get 43
      f64.mul
      local.get 20
      local.get 41
      local.get 41
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 20
      local.get 20
      f64.mul
      f64.sub
      f64.add
      local.tee 41
      local.get 41
      f64.mul
      local.set 6
      local.get 20
      local.get 43
      local.get 43
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 20
      local.get 20
      f64.mul
      local.set 28
      local.get 7
      local.get 7
      local.get 7
      local.get 7
      local.get 8
      local.get 29
      f64.sub
      f64.add
      local.tee 43
      local.get 43
      f64.mul
      local.get 21
      local.get 40
      local.get 40
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 21
      local.get 21
      f64.mul
      f64.sub
      f64.add
      local.tee 40
      local.get 40
      f64.mul
      local.get 43
      local.get 43
      f64.add
      local.get 21
      f64.mul
      local.get 1
      f64.add
      local.tee 21
      local.get 21
      f64.mul
      f64.sub
      f64.add
      local.tee 43
      local.get 43
      f64.mul
      local.get 21
      local.get 40
      local.get 40
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 21
      local.get 21
      f64.mul
      f64.sub
      f64.add
      local.tee 40
      local.get 40
      f64.mul
      local.set 8
      local.get 21
      local.get 43
      local.get 43
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 21
      local.get 21
      f64.mul
      local.set 29
      local.get 9
      local.get 9
      local.get 9
      local.get 9
      local.get 10
      local.get 30
      f64.sub
      f64.add
      local.tee 43
      local.get 43
      f64.mul
      local.get 22
      local.get 39
      local.get 39
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 22
      local.get 22
      f64.mul
      f64.sub
      f64.add
      local.tee 39
      local.get 39
      f64.mul
      local.get 43
      local.get 43
      f64.add
      local.get 22
      f64.mul
      local.get 1
      f64.add
      local.tee 22
      local.get 22
      f64.mul
      f64.sub
      f64.add
      local.tee 43
      local.get 43
      f64.mul
      local.get 22
      local.get 39
      local.get 39
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 22
      local.get 22
      f64.mul
      f64.sub
      f64.add
      local.tee 39
      local.get 39
      f64.mul
      local.set 10
      local.get 22
      local.get 43
      local.get 43
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 22
      local.get 22
      f64.mul
      local.set 30
      local.get 11
      local.get 12
      local.get 31
      f64.sub
      f64.add
      local.tee 43
      local.get 43
      f64.add
      local.get 23
      local.get 38
      local.get 38
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 38
      f64.mul
      local.get 1
      f64.add
      local.tee 23
      local.get 11
      local.get 43
      local.get 43
      f64.mul
      local.get 38
      local.get 38
      f64.mul
      f64.sub
      f64.add
      local.tee 38
      local.get 38
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 12
      local.get 11
      local.get 38
      local.get 38
      f64.mul
      local.get 23
      local.get 23
      f64.mul
      f64.sub
      f64.add
      local.tee 38
      local.get 38
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 23
      local.get 23
      f64.mul
      local.set 31
      local.get 13
      local.get 14
      local.get 32
      f64.sub
      f64.add
      local.tee 43
      local.get 43
      f64.add
      local.get 24
      local.get 37
      local.get 37
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 37
      f64.mul
      local.get 1
      f64.add
      local.tee 24
      local.get 13
      local.get 43
      local.get 43
      f64.mul
      local.get 37
      local.get 37
      f64.mul
      f64.sub
      f64.add
      local.tee 37
      local.get 37
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 14
      local.get 13
      local.get 37
      local.get 37
      f64.mul
      local.get 24
      local.get 24
      f64.mul
      f64.sub
      f64.add
      local.tee 37
      local.get 37
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 24
      local.get 24
      f64.mul
      local.set 32
      local.get 15
      local.get 16
      local.get 33
      f64.sub
      f64.add
      local.tee 43
      local.get 43
      f64.add
      local.get 25
      local.get 36
      local.get 36
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 36
      f64.mul
      local.get 1
      f64.add
      local.tee 25
      local.get 15
      local.get 43
      local.get 43
      f64.mul
      local.get 36
      local.get 36
      f64.mul
      f64.sub
      f64.add
      local.tee 36
      local.get 36
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 16
      local.get 15
      local.get 36
      local.get 36
      f64.mul
      local.get 25
      local.get 25
      f64.mul
      f64.sub
      f64.add
      local.tee 36
      local.get 36
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 25
      local.get 25
      f64.mul
      local.set 33
      local.get 17
      local.get 18
      local.get 34
      f64.sub
      f64.add
      local.tee 43
      local.get 43
      f64.add
      local.get 26
      local.get 35
      local.get 35
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 35
      f64.mul
      local.get 1
      f64.add
      local.tee 26
      local.get 17
      local.get 43
      local.get 43
      f64.mul
      local.get 35
      local.get 35
      f64.mul
      f64.sub
      f64.add
      local.tee 35
      local.get 35
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 43
      local.get 17
      local.get 35
      local.get 35
      f64.mul
      local.get 26
      local.get 26
      f64.mul
      f64.sub
      f64.add
      local.tee 35
      local.get 35
      f64.add
      f64.mul
      local.get 1
      f64.add
      local.tee 26
      local.get 26
      f64.mul
      local.set 34
      local.get 11
      local.get 38
      local.get 38
      f64.mul
      local.get 12
      local.get 12
      f64.mul
      f64.sub
      f64.add
      local.tee 38
      local.get 38
      f64.mul
      local.set 12
      local.get 13
      local.get 37
      local.get 37
      f64.mul
      local.get 14
      local.get 14
      f64.mul
      f64.sub
      f64.add
      local.tee 37
      local.get 37
      f64.mul
      local.set 14
      local.get 15
      local.get 36
      local.get 36
      f64.mul
      local.get 16
      local.get 16
      f64.mul
      f64.sub
      f64.add
      local.tee 36
      local.get 36
      f64.mul
      local.set 16
      local.get 17
      local.get 35
      local.get 35
      f64.mul
      local.get 43
      local.get 43
      f64.mul
      f64.sub
      f64.add
      local.tee 35
      local.get 35
      f64.mul
      local.set 18
      block  ;; label = @2
        local.get 2
        br_if 0 (;@2;)
        local.get 31
        local.get 12
        f64.add
        f64.const 0x1p+2 (;=4;)
        f64.gt
        i32.eqz
        br_if 0 (;@2;)
        local.get 32
        local.get 14
        f64.add
        f64.const 0x1p+2 (;=4;)
        f64.gt
        i32.eqz
        br_if 0 (;@2;)
        local.get 34
        local.get 18
        f64.add
        f64.const 0x1p+2 (;=4;)
        f64.gt
        i32.eqz
        br_if 0 (;@2;)
        local.get 33
        local.get 16
        f64.add
        f64.const 0x1p+2 (;=4;)
        f64.gt
        i32.eqz
        br_if 0 (;@2;)
        local.get 27
        local.get 4
        f64.add
        f64.const 0x1p+2 (;=4;)
        f64.gt
        i32.eqz
        br_if 0 (;@2;)
        local.get 28
        local.get 6
        f64.add
        f64.const 0x1p+2 (;=4;)
        f64.gt
        i32.eqz
        br_if 0 (;@2;)
        local.get 30
        local.get 10
        f64.add
        f64.const 0x1p+2 (;=4;)
        f64.gt
        i32.eqz
        br_if 0 (;@2;)
        local.get 29
        local.get 8
        f64.add
        f64.const 0x1p+2 (;=4;)
        f64.gt
        i32.eqz
        br_if 0 (;@2;)
        i32.const 0
        return
      end
      local.get 0
      i32.const -1
      i32.add
      local.tee 0
      br_if 0 (;@1;)
    end
    local.get 25
    local.get 36
    local.get 36
    f64.add
    f64.mul
    local.get 1
    f64.add
    local.tee 36
    local.get 36
    f64.mul
    local.get 15
    local.get 16
    local.get 33
    f64.sub
    f64.add
    local.tee 15
    local.get 15
    f64.mul
    f64.add
    f64.const 0x1p+2 (;=4;)
    f64.le
    i32.const 1
    i32.shl
    local.get 26
    local.get 35
    local.get 35
    f64.add
    f64.mul
    local.get 1
    f64.add
    local.tee 15
    local.get 15
    f64.mul
    local.get 17
    local.get 18
    local.get 34
    f64.sub
    f64.add
    local.tee 15
    local.get 15
    f64.mul
    f64.add
    f64.const 0x1p+2 (;=4;)
    f64.le
    i32.or
    local.get 24
    local.get 37
    local.get 37
    f64.add
    f64.mul
    local.get 1
    f64.add
    local.tee 15
    local.get 15
    f64.mul
    local.get 13
    local.get 14
    local.get 32
    f64.sub
    f64.add
    local.tee 13
    local.get 13
    f64.mul
    f64.add
    f64.const 0x1p+2 (;=4;)
    f64.le
    i32.const 2
    i32.shl
    i32.or
    local.get 23
    local.get 38
    local.get 38
    f64.add
    f64.mul
    local.get 1
    f64.add
    local.tee 13
    local.get 13
    f64.mul
    local.get 11
    local.get 12
    local.get 31
    f64.sub
    f64.add
    local.tee 11
    local.get 11
    f64.mul
    f64.add
    f64.const 0x1p+2 (;=4;)
    f64.le
    i32.const 3
    i32.shl
    i32.or
    local.get 22
    local.get 39
    local.get 39
    f64.add
    f64.mul
    local.get 1
    f64.add
    local.tee 11
    local.get 11
    f64.mul
    local.get 9
    local.get 10
    local.get 30
    f64.sub
    f64.add
    local.tee 9
    local.get 9
    f64.mul
    f64.add
    f64.const 0x1p+2 (;=4;)
    f64.le
    i32.const 4
    i32.shl
    i32.or
    local.get 21
    local.get 40
    local.get 40
    f64.add
    f64.mul
    local.get 1
    f64.add
    local.tee 9
    local.get 9
    f64.mul
    local.get 7
    local.get 8
    local.get 29
    f64.sub
    f64.add
    local.tee 7
    local.get 7
    f64.mul
    f64.add
    f64.const 0x1p+2 (;=4;)
    f64.le
    i32.const 5
    i32.shl
    i32.or
    local.get 20
    local.get 41
    local.get 41
    f64.add
    f64.mul
    local.get 1
    f64.add
    local.tee 7
    local.get 7
    f64.mul
    local.get 5
    local.get 6
    local.get 28
    f64.sub
    f64.add
    local.tee 5
    local.get 5
    f64.mul
    f64.add
    f64.const 0x1p+2 (;=4;)
    f64.le
    i32.const 6
    i32.shl
    i32.or
    local.get 19
    local.get 42
    local.get 42
    f64.add
    f64.mul
    local.get 1
    f64.add
    local.tee 1
    local.get 1
    f64.mul
    local.get 3
    local.get 4
    local.get 27
    f64.sub
    f64.add
    local.tee 1
    local.get 1
    f64.mul
    f64.add
    f64.const 0x1p+2 (;=4;)
    f64.le
    i32.const 7
    i32.shl
    i32.or)
  (func $_ZN83_$LT$rayon_core..job..StackJob$LT$L$C$F$C$R$GT$$u20$as$u20$rayon_core..job..Job$GT$7execute17h8a40026598c18a69E (type 0) (param i32)
    (local i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 64
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 0
    i32.load
    local.set 2
    local.get 0
    i32.const 0
    i32.store
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 2
          i32.eqz
          br_if 0 (;@3;)
          local.get 1
          i32.const 12
          i32.add
          local.get 0
          i32.const 12
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 20
          i32.add
          local.get 0
          i32.const 20
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 28
          i32.add
          local.get 0
          i32.const 28
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 36
          i32.add
          local.get 0
          i32.const 36
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 44
          i32.add
          local.get 0
          i32.const 44
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 52
          i32.add
          local.get 0
          i32.const 52
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          local.get 2
          i32.store
          local.get 1
          local.get 0
          i64.load offset=4 align=4
          i64.store offset=4 align=4
          i32.const 0
          i32.load offset=1058996
          local.tee 2
          i32.eqz
          br_if 1 (;@2;)
          local.get 1
          local.get 2
          call $_ZN10rayon_core4join12join_context28_$u7b$$u7b$closure$u7d$$u7d$17h8fe34789bad32535E
          block  ;; label = @4
            local.get 0
            i32.load offset=60
            i32.const 2
            i32.lt_u
            br_if 0 (;@4;)
            local.get 0
            i32.load offset=64
            local.set 3
            block  ;; label = @5
              local.get 0
              i32.load offset=68
              local.tee 2
              i32.load
              local.tee 4
              i32.eqz
              br_if 0 (;@5;)
              local.get 3
              local.get 4
              call_indirect (type 0)
            end
            local.get 2
            i32.load offset=4
            local.tee 4
            i32.eqz
            br_if 0 (;@4;)
            local.get 3
            local.get 4
            local.get 2
            i32.load offset=8
            call $__rust_dealloc
          end
          local.get 0
          i32.const 1
          i32.store offset=60
          local.get 0
          i32.load offset=72
          i32.load
          local.set 2
          block  ;; label = @4
            block  ;; label = @5
              local.get 0
              i32.load8_u offset=84
              br_if 0 (;@5;)
              local.get 0
              i32.load offset=76
              local.set 3
              local.get 0
              i32.const 3
              i32.store offset=76
              local.get 3
              i32.const 2
              i32.ne
              br_if 1 (;@4;)
              local.get 2
              i32.const 64
              i32.add
              local.get 0
              i32.load offset=80
              call $_ZN10rayon_core8registry8Registry26notify_worker_latch_is_set17h0c173997f1598668E
              br 1 (;@4;)
            end
            local.get 2
            local.get 2
            i32.load
            local.tee 3
            i32.const 1
            i32.add
            i32.store
            local.get 3
            i32.const 0
            i32.lt_s
            br_if 3 (;@1;)
            local.get 0
            i32.load offset=76
            local.set 3
            local.get 0
            i32.const 3
            i32.store offset=76
            local.get 1
            local.get 2
            i32.store offset=60
            block  ;; label = @5
              local.get 3
              i32.const 2
              i32.ne
              br_if 0 (;@5;)
              local.get 2
              i32.const 64
              i32.add
              local.get 0
              i32.load offset=80
              call $_ZN10rayon_core8registry8Registry26notify_worker_latch_is_set17h0c173997f1598668E
            end
            local.get 1
            i32.load offset=60
            local.tee 0
            local.get 0
            i32.load
            local.tee 0
            i32.const -1
            i32.add
            i32.store
            local.get 0
            i32.const 1
            i32.ne
            br_if 0 (;@4;)
            local.get 1
            i32.const 60
            i32.add
            call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h76aa42d8034d2782E
          end
          local.get 1
          i32.const 64
          i32.add
          global.set $__stack_pointer
          return
        end
        i32.const 1049248
        call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
        unreachable
      end
      i32.const 1049060
      i32.const 54
      i32.const 1049232
      call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
    end
    unreachable)
  (func $_ZN83_$LT$rayon_core..job..StackJob$LT$L$C$F$C$R$GT$$u20$as$u20$rayon_core..job..Job$GT$7execute17hf4bf52d52279e47bE (type 0) (param i32)
    (local i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 96
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 0
    i32.load offset=4
    local.set 2
    local.get 0
    i32.const 0
    i32.store offset=4
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 2
          i32.eqz
          br_if 0 (;@3;)
          local.get 1
          i32.const 20
          i32.add
          local.get 0
          i32.const 16
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 28
          i32.add
          local.get 0
          i32.const 24
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 36
          i32.add
          local.get 0
          i32.const 32
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 44
          i32.add
          local.get 0
          i32.const 40
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 52
          i32.add
          local.get 0
          i32.const 48
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 60
          i32.add
          local.get 0
          i32.const 56
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          local.get 2
          i32.store offset=8
          local.get 1
          local.get 0
          i64.load offset=8 align=4
          i64.store offset=12 align=4
          i32.const 0
          i32.load offset=1058996
          local.tee 2
          i32.eqz
          br_if 1 (;@2;)
          local.get 1
          i32.const 8
          i32.add
          local.get 2
          call $_ZN10rayon_core4join12join_context28_$u7b$$u7b$closure$u7d$$u7d$17h8fe34789bad32535E
          block  ;; label = @4
            local.get 0
            i32.load offset=64
            i32.const 2
            i32.lt_u
            br_if 0 (;@4;)
            local.get 0
            i32.load offset=68
            local.set 3
            block  ;; label = @5
              local.get 0
              i32.load offset=72
              local.tee 2
              i32.load
              local.tee 4
              i32.eqz
              br_if 0 (;@5;)
              local.get 3
              local.get 4
              call_indirect (type 0)
            end
            local.get 2
            i32.load offset=4
            local.tee 4
            i32.eqz
            br_if 0 (;@4;)
            local.get 3
            local.get 4
            local.get 2
            i32.load offset=8
            call $__rust_dealloc
          end
          local.get 0
          i32.const 1
          i32.store offset=64
          local.get 1
          local.get 0
          i32.load
          local.tee 0
          i32.load8_u
          local.tee 2
          i32.store8 offset=71
          local.get 0
          i32.const 1
          i32.store8
          local.get 2
          i32.const 1
          i32.eq
          br_if 2 (;@1;)
          local.get 0
          i32.const 256
          i32.store16 align=1
          local.get 1
          i32.const 96
          i32.add
          global.set $__stack_pointer
          return
        end
        i32.const 1049248
        call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
        unreachable
      end
      i32.const 1049060
      i32.const 54
      i32.const 1049216
      call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
      unreachable
    end
    local.get 1
    i64.const 0
    i64.store offset=84 align=4
    local.get 1
    i64.const 17179869185
    i64.store offset=76 align=4
    local.get 1
    i32.const 1049480
    i32.store offset=72
    i32.const 0
    local.get 1
    i32.const 71
    i32.add
    i32.const 1049488
    local.get 1
    i32.const 72
    i32.add
    i32.const 1049584
    call $_ZN4core9panicking13assert_failed17hf847e9f68e665e06E
    unreachable)
  (func $mandelbrot (type 1)
    (local i32 i32)
    global.get $__stack_pointer
    i32.const 80
    i32.sub
    local.tee 0
    global.set $__stack_pointer
    local.get 0
    i32.const 200
    i32.store offset=12
    local.get 0
    i64.const 4576918229304087675
    i64.store offset=16
    local.get 0
    i64.const 107374182400
    i64.store offset=44 align=4
    local.get 0
    local.get 0
    i32.const 16
    i32.add
    i32.store offset=40
    local.get 0
    i32.const 28
    i32.add
    local.get 0
    i32.const 40
    i32.add
    i32.const 1049352
    call $_ZN98_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter..SpecFromIter$LT$T$C$I$GT$$GT$9from_iter17h3a1156df5a7789eeE
    i32.const 0
    i32.load8_u offset=1058985
    drop
    block  ;; label = @1
      i32.const 5000
      i32.const 1
      call $__rust_alloc_zeroed
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 0
      i32.const 16
      i32.add
      i32.store offset=60
      local.get 0
      local.get 0
      i32.const 12
      i32.add
      i32.store offset=56
      local.get 0
      local.get 0
      i32.const 28
      i32.add
      i32.store offset=52
      local.get 0
      i32.const 0
      i32.store offset=76
      local.get 0
      i64.const 107374187400
      i64.store offset=68 align=4
      local.get 0
      local.get 1
      i32.store offset=64
      i32.const 200
      i32.const 0
      call $_ZN10rayon_core19current_num_threads17h7db6c4811dab6784E
      i32.const 1
      local.get 0
      i32.const 64
      i32.add
      local.get 0
      i32.const 52
      i32.add
      call $_ZN5rayon4iter8plumbing24bridge_producer_consumer6helper17h124c3a64ae04bff2E
      local.get 1
      i32.const 5000
      i32.const 1
      call $__rust_dealloc
      block  ;; label = @2
        local.get 0
        i32.load offset=28
        local.tee 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 0
        i32.load offset=32
        local.get 1
        i32.const 6
        i32.shl
        i32.const 32
        call $__rust_dealloc
      end
      local.get 0
      i32.const 80
      i32.add
      global.set $__stack_pointer
      return
    end
    i32.const 1
    i32.const 5000
    i32.const 1049384
    call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
    unreachable)
  (func $_ZN10mandelbrot4main17haebc31b42b6c82b6E (type 1)
    call $mandelbrot)
  (func $main (type 4) (param i32 i32) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 6
    i32.store offset=12
    local.get 2
    i32.const 12
    i32.add
    i32.const 1048728
    local.get 0
    local.get 1
    i32.const 0
    call $_ZN3std2rt19lang_start_internal17hdc6030aca1dd7348E
    local.set 1
    local.get 2
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 1)
  (func $_ZN10rayon_core4join12join_context28_$u7b$$u7b$closure$u7d$$u7d$17h8fe34789bad32535E.llvm.2793606751137166678 (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 96
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 16
    i32.add
    i32.const 8
    i32.add
    local.get 0
    i32.const 8
    i32.add
    i64.load align=4
    i64.store
    local.get 2
    i32.const 16
    i32.add
    i32.const 16
    i32.add
    local.get 0
    i32.const 16
    i32.add
    i64.load align=4
    i64.store
    local.get 2
    i32.const 16
    i32.add
    i32.const 24
    i32.add
    local.get 0
    i32.const 24
    i32.add
    i64.load align=4
    i64.store
    local.get 2
    i32.const 0
    i32.store8 offset=72
    local.get 2
    i32.const 0
    i32.store offset=64
    local.get 2
    i32.const 0
    i32.store offset=48
    local.get 2
    local.get 1
    i32.load offset=136
    i32.store offset=68
    local.get 2
    local.get 1
    i32.const 140
    i32.add
    i32.store offset=60
    local.get 2
    local.get 0
    i64.load align=4
    i64.store offset=16
    local.get 1
    i32.const 144
    i32.add
    local.set 3
    block  ;; label = @1
      local.get 1
      i32.load offset=144
      local.tee 4
      i32.load offset=132
      local.tee 5
      local.get 4
      i32.load offset=128
      i32.sub
      local.tee 6
      local.get 1
      i32.load offset=152
      local.tee 4
      i32.lt_s
      br_if 0 (;@1;)
      local.get 3
      local.get 4
      i32.const 1
      i32.shl
      call $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$6resize17h3d4716eead8d3956E.llvm.6352068949975026865
      local.get 1
      i32.load offset=152
      local.set 4
    end
    local.get 1
    i32.load offset=148
    local.get 4
    i32.const -1
    i32.add
    local.get 5
    i32.and
    i32.const 3
    i32.shl
    i32.add
    local.tee 4
    i32.const 5
    i32.store
    local.get 4
    local.get 2
    i32.const 16
    i32.add
    i32.store offset=4
    local.get 1
    i32.load offset=144
    local.get 5
    i32.const 1
    i32.add
    i32.store offset=132
    local.get 1
    i32.load offset=140
    local.set 4
    block  ;; label = @1
      loop  ;; label = @2
        block  ;; label = @3
          local.get 4
          i32.load offset=248
          local.tee 5
          i32.const 65536
          i32.and
          i32.eqz
          br_if 0 (;@3;)
          local.get 5
          local.set 7
          br 2 (;@1;)
        end
        local.get 4
        local.get 5
        i32.const 65536
        i32.or
        local.tee 7
        local.get 4
        i32.load offset=248
        local.tee 8
        local.get 8
        local.get 5
        i32.eq
        select
        i32.store offset=248
        local.get 8
        local.get 5
        i32.ne
        br_if 0 (;@2;)
      end
    end
    block  ;; label = @1
      local.get 7
      i32.const 255
      i32.and
      local.tee 5
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 6
        i32.const 0
        i32.gt_s
        br_if 0 (;@2;)
        local.get 7
        i32.const 8
        i32.shr_u
        i32.const 255
        i32.and
        local.get 5
        i32.ne
        br_if 1 (;@1;)
      end
      local.get 4
      i32.const 236
      i32.add
      i32.const 1
      call $_ZN10rayon_core5sleep5Sleep16wake_any_threads17h17244bbca7ed393dE
    end
    local.get 0
    i32.load offset=36
    local.set 5
    local.get 0
    i32.load offset=32
    local.set 4
    local.get 2
    i32.const 88
    i32.add
    local.get 0
    i32.const 48
    i32.add
    i64.load align=4
    i64.store
    local.get 2
    local.get 0
    i64.load offset=40 align=4
    i64.store offset=80
    local.get 4
    i32.load
    i32.const 0
    local.get 5
    i32.load
    local.get 5
    i32.load offset=4
    local.get 2
    i32.const 80
    i32.add
    local.get 0
    i32.load offset=56
    call $_ZN5rayon4iter8plumbing24bridge_producer_consumer6helper17h124c3a64ae04bff2E
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        i32.load offset=64
        i32.const 3
        i32.eq
        br_if 0 (;@2;)
        local.get 1
        i32.const 160
        i32.add
        local.set 4
        block  ;; label = @3
          block  ;; label = @4
            loop  ;; label = @5
              local.get 2
              i32.const 8
              i32.add
              local.get 3
              call $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$3pop17h1ff9952805e7f2a6E
              block  ;; label = @6
                block  ;; label = @7
                  local.get 2
                  i32.load offset=8
                  local.tee 5
                  i32.eqz
                  br_if 0 (;@7;)
                  local.get 2
                  i32.load offset=12
                  local.set 8
                  br 1 (;@6;)
                end
                loop  ;; label = @7
                  local.get 2
                  i32.const 80
                  i32.add
                  local.get 4
                  call $_ZN15crossbeam_deque5deque16Stealer$LT$T$GT$5steal17h92560776915b651eE
                  local.get 2
                  i32.load offset=80
                  local.tee 5
                  i32.const 2
                  i32.eq
                  br_if 0 (;@7;)
                end
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 5
                    br_table 0 (;@8;) 1 (;@7;) 0 (;@8;)
                  end
                  local.get 2
                  i32.load offset=64
                  i32.const 3
                  i32.ne
                  br_if 3 (;@4;)
                  br 5 (;@2;)
                end
                local.get 2
                i32.load offset=88
                local.set 8
                local.get 2
                i32.load offset=84
                local.set 5
              end
              block  ;; label = @6
                block  ;; label = @7
                  local.get 5
                  i32.const 5
                  i32.ne
                  br_if 0 (;@7;)
                  local.get 2
                  i32.const 16
                  i32.add
                  local.get 8
                  i32.eq
                  br_if 1 (;@6;)
                end
                local.get 8
                local.get 5
                call_indirect (type 0)
                local.get 2
                i32.load offset=64
                i32.const 3
                i32.ne
                br_if 1 (;@5;)
                br 4 (;@2;)
              end
            end
            local.get 2
            i32.load offset=16
            local.tee 8
            i32.eqz
            br_if 1 (;@3;)
            local.get 2
            i32.load offset=56
            local.set 4
            local.get 2
            i32.load offset=52
            local.set 3
            local.get 2
            i32.load offset=48
            local.set 7
            local.get 2
            i32.load offset=44
            local.set 0
            local.get 2
            i32.load offset=24
            local.set 5
            local.get 2
            i32.load offset=20
            local.set 1
            local.get 2
            i32.const 88
            i32.add
            local.get 2
            i32.const 36
            i32.add
            i64.load align=4
            i64.store
            local.get 2
            local.get 2
            i64.load offset=28 align=4
            i64.store offset=80
            local.get 8
            i32.load
            local.get 1
            i32.load
            i32.sub
            i32.const 0
            local.get 5
            i32.load
            local.get 5
            i32.load offset=4
            local.get 2
            i32.const 80
            i32.add
            local.get 0
            call $_ZN5rayon4iter8plumbing24bridge_producer_consumer6helper17h124c3a64ae04bff2E
            local.get 7
            i32.const 2
            i32.lt_u
            br_if 3 (;@1;)
            block  ;; label = @5
              local.get 4
              i32.load
              local.tee 5
              i32.eqz
              br_if 0 (;@5;)
              local.get 3
              local.get 5
              call_indirect (type 0)
            end
            local.get 4
            i32.load offset=4
            local.tee 5
            i32.eqz
            br_if 3 (;@1;)
            local.get 3
            local.get 5
            local.get 4
            i32.load offset=8
            call $__rust_dealloc
            br 3 (;@1;)
          end
          local.get 1
          local.get 2
          i32.const 64
          i32.add
          call $_ZN10rayon_core8registry12WorkerThread15wait_until_cold17hdc4a872cdf442250E
          br 1 (;@2;)
        end
        i32.const 1049044
        call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
        unreachable
      end
      local.get 2
      i32.load offset=48
      local.tee 5
      i32.const 1
      i32.eq
      br_if 0 (;@1;)
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 5
            br_table 1 (;@3;) 0 (;@4;) 2 (;@2;) 1 (;@3;)
          end
          unreachable
        end
        i32.const 1048888
        i32.const 40
        i32.const 1049028
        call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
        unreachable
      end
      local.get 2
      i32.load offset=52
      local.get 2
      i32.load offset=56
      call $_ZN10rayon_core6unwind16resume_unwinding17h9301afba3ac4692cE
      unreachable
    end
    local.get 2
    i32.const 96
    i32.add
    global.set $__stack_pointer)
  (func $_ZN10rayon_core8registry8Registry14in_worker_cold17h3a0b00e35b93607aE.llvm.2793606751137166678 (type 2) (param i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 80
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      i32.const 0
      i32.load8_u offset=1058992
      br_if 0 (;@1;)
      i32.const 0
      i32.const 1
      i32.store16 offset=1058992 align=1
      i32.const 0
      i32.const 0
      i32.store8 offset=1058994
    end
    local.get 2
    i32.const 16
    i32.add
    local.get 1
    i32.const 8
    i32.add
    i64.load align=4
    i64.store align=4
    local.get 2
    i32.const 24
    i32.add
    local.get 1
    i32.const 16
    i32.add
    i64.load align=4
    i64.store align=4
    local.get 2
    i32.const 32
    i32.add
    local.get 1
    i32.const 24
    i32.add
    i64.load align=4
    i64.store align=4
    local.get 2
    i32.const 40
    i32.add
    local.get 1
    i32.const 32
    i32.add
    i64.load align=4
    i64.store align=4
    local.get 2
    i32.const 48
    i32.add
    local.get 1
    i32.const 40
    i32.add
    i64.load align=4
    i64.store align=4
    local.get 2
    i32.const 56
    i32.add
    local.get 1
    i32.const 48
    i32.add
    i64.load align=4
    i64.store align=4
    local.get 2
    i32.const 64
    i32.add
    local.get 1
    i32.const 56
    i32.add
    i32.load
    i32.store
    local.get 2
    i32.const 1058993
    i32.store offset=4
    local.get 2
    i32.const 0
    i32.store offset=68
    local.get 2
    local.get 1
    i64.load align=4
    i64.store offset=8 align=4
    local.get 0
    i32.const 7
    local.get 2
    i32.const 4
    i32.add
    call $_ZN10rayon_core8registry8Registry6inject17h51e937ac645609bdE
    local.get 2
    i32.load offset=4
    call $_ZN10rayon_core5latch9LockLatch14wait_and_reset17hd076fbaf8601b838E
    block  ;; label = @1
      local.get 2
      i32.load offset=68
      local.tee 1
      i32.const 1
      i32.eq
      br_if 0 (;@1;)
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 1
            br_table 1 (;@3;) 0 (;@4;) 2 (;@2;) 1 (;@3;)
          end
          unreachable
        end
        i32.const 1048888
        i32.const 40
        i32.const 1049028
        call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
        unreachable
      end
      local.get 2
      i32.load offset=72
      local.get 2
      i32.load offset=76
      call $_ZN10rayon_core6unwind16resume_unwinding17h9301afba3ac4692cE
      unreachable
    end
    local.get 2
    i32.const 80
    i32.add
    global.set $__stack_pointer)
  (func $_ZN10rayon_core8registry8Registry15in_worker_cross17h4ec1b8df6cedc729E.llvm.2793606751137166678 (type 3) (param i32 i32 i32)
    (local i32 i32)
    global.get $__stack_pointer
    i32.const 96
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 1
    i32.load offset=136
    local.set 4
    local.get 3
    i32.const 8
    i32.add
    i32.const 8
    i32.add
    local.get 2
    i32.const 8
    i32.add
    i64.load align=4
    i64.store
    local.get 3
    i32.const 8
    i32.add
    i32.const 16
    i32.add
    local.get 2
    i32.const 16
    i32.add
    i64.load align=4
    i64.store
    local.get 3
    i32.const 8
    i32.add
    i32.const 24
    i32.add
    local.get 2
    i32.const 24
    i32.add
    i64.load align=4
    i64.store
    local.get 3
    i32.const 8
    i32.add
    i32.const 32
    i32.add
    local.get 2
    i32.const 32
    i32.add
    i64.load align=4
    i64.store
    local.get 3
    i32.const 8
    i32.add
    i32.const 40
    i32.add
    local.get 2
    i32.const 40
    i32.add
    i64.load align=4
    i64.store
    local.get 3
    i32.const 8
    i32.add
    i32.const 48
    i32.add
    local.get 2
    i32.const 48
    i32.add
    i64.load align=4
    i64.store
    local.get 3
    i32.const 8
    i32.add
    i32.const 56
    i32.add
    local.get 2
    i32.const 56
    i32.add
    i32.load
    i32.store
    local.get 3
    i32.const 1
    i32.store8 offset=92
    local.get 3
    local.get 4
    i32.store offset=88
    local.get 3
    i32.const 0
    i32.store offset=84
    local.get 3
    local.get 1
    i32.const 140
    i32.add
    i32.store offset=80
    local.get 3
    local.get 2
    i64.load align=4
    i64.store offset=8
    local.get 3
    i32.const 0
    i32.store offset=68
    local.get 0
    i32.const 8
    local.get 3
    i32.const 8
    i32.add
    call $_ZN10rayon_core8registry8Registry6inject17h51e937ac645609bdE
    block  ;; label = @1
      local.get 3
      i32.load offset=84
      i32.const 3
      i32.eq
      br_if 0 (;@1;)
      local.get 1
      local.get 3
      i32.const 84
      i32.add
      call $_ZN10rayon_core8registry12WorkerThread15wait_until_cold17hdc4a872cdf442250E
    end
    block  ;; label = @1
      local.get 3
      i32.load offset=68
      local.tee 2
      i32.const 1
      i32.eq
      br_if 0 (;@1;)
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 2
            br_table 1 (;@3;) 0 (;@4;) 2 (;@2;) 1 (;@3;)
          end
          unreachable
        end
        i32.const 1048888
        i32.const 40
        i32.const 1049028
        call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
        unreachable
      end
      local.get 3
      i32.load offset=72
      local.get 3
      i32.load offset=76
      call $_ZN10rayon_core6unwind16resume_unwinding17h9301afba3ac4692cE
      unreachable
    end
    local.get 3
    i32.const 96
    i32.add
    global.set $__stack_pointer)
  (func $_ZN15crossbeam_epoch8deferred8Deferred3new4call17hba3722852837b63cE.llvm.2793606751137166678 (type 0) (param i32)
    (local i32)
    block  ;; label = @1
      local.get 0
      i32.load
      i32.const -4
      i32.and
      local.tee 0
      i32.load offset=4
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.load
      local.get 1
      i32.const 3
      i32.shl
      i32.const 4
      call $__rust_dealloc
    end
    local.get 0
    i32.const 8
    i32.const 4
    call $__rust_dealloc)
  (func $_ZN3std3sys9backtrace28__rust_begin_short_backtrace17hbe4fecc903c07c99E (type 0) (param i32)
    local.get 0
    call_indirect (type 1))
  (func $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h02cb3d0998e89334E (type 4) (param i32 i32) (result i32)
    (local i32)
    local.get 0
    i32.load
    local.set 0
    block  ;; label = @1
      local.get 1
      i32.load offset=28
      local.tee 2
      i32.const 16
      i32.and
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 2
        i32.const 32
        i32.and
        br_if 0 (;@2;)
        local.get 0
        local.get 1
        call $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17he1d3bba66865ae66E
        return
      end
      local.get 0
      local.get 1
      call $_ZN4core3fmt3num53_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$i32$GT$3fmt17h2e92699d27a37844E
      return
    end
    local.get 0
    local.get 1
    call $_ZN4core3fmt3num53_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$i32$GT$3fmt17hee4425a51b6b9c20E)
  (func $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17ha4277154763c2c00E (type 4) (param i32 i32) (result i32)
    local.get 0
    i32.load
    local.get 1
    call $_ZN43_$LT$bool$u20$as$u20$core..fmt..Display$GT$3fmt17h90c5085ed9b38d31E)
  (func $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h76aa42d8034d2782E (type 0) (param i32)
    (local i32 i32 i32 i32)
    block  ;; label = @1
      local.get 0
      i32.load
      local.tee 1
      i32.load offset=260
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.load offset=256
      local.set 0
      loop  ;; label = @2
        local.get 0
        i32.load
        local.tee 3
        local.get 3
        i32.load
        local.tee 3
        i32.const -1
        i32.add
        i32.store
        block  ;; label = @3
          local.get 3
          i32.const 1
          i32.ne
          br_if 0 (;@3;)
          local.get 0
          call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hc628a5a1c0300638E
        end
        local.get 0
        i32.const 16
        i32.add
        local.set 0
        local.get 2
        i32.const -1
        i32.add
        local.tee 2
        br_if 0 (;@2;)
      end
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=252
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.load offset=256
      local.get 0
      i32.const 4
      i32.shl
      i32.const 4
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=236
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.load offset=240
      local.get 0
      i32.const 6
      i32.shl
      i32.const 64
      call $__rust_dealloc
    end
    local.get 1
    i32.load offset=68
    local.set 2
    block  ;; label = @1
      local.get 1
      i32.load offset=64
      i32.const -2
      i32.and
      local.tee 0
      local.get 1
      i32.load offset=128
      i32.const -2
      i32.and
      local.tee 3
      i32.eq
      br_if 0 (;@1;)
      loop  ;; label = @2
        block  ;; label = @3
          local.get 0
          i32.const 126
          i32.and
          i32.const 126
          i32.ne
          br_if 0 (;@3;)
          local.get 2
          i32.load
          local.set 4
          local.get 2
          i32.const 760
          i32.const 4
          call $__rust_dealloc
          local.get 4
          local.set 2
        end
        local.get 3
        local.get 0
        i32.const 2
        i32.add
        local.tee 0
        i32.ne
        br_if 0 (;@2;)
      end
    end
    local.get 2
    i32.const 760
    i32.const 4
    call $__rust_dealloc
    block  ;; label = @1
      local.get 1
      i32.load offset=204
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.load offset=200
      local.set 0
      loop  ;; label = @2
        local.get 0
        i32.load
        local.tee 3
        local.get 3
        i32.load
        local.tee 3
        i32.const -1
        i32.add
        i32.store
        block  ;; label = @3
          local.get 3
          i32.const 1
          i32.ne
          br_if 0 (;@3;)
          local.get 0
          call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hc628a5a1c0300638E
        end
        local.get 0
        i32.const 16
        i32.add
        local.set 0
        local.get 2
        i32.const -1
        i32.add
        local.tee 2
        br_if 0 (;@2;)
      end
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=196
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.load offset=200
      local.get 0
      i32.const 4
      i32.shl
      i32.const 4
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=208
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 1
        i32.load offset=212
        local.tee 2
        i32.load
        local.tee 3
        i32.eqz
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        call_indirect (type 0)
      end
      local.get 2
      i32.load offset=4
      local.tee 3
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 3
      local.get 2
      i32.load offset=8
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=216
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 1
        i32.load offset=220
        local.tee 2
        i32.load
        local.tee 3
        i32.eqz
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        call_indirect (type 0)
      end
      local.get 2
      i32.load offset=4
      local.tee 3
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 3
      local.get 2
      i32.load offset=8
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=224
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 1
        i32.load offset=228
        local.tee 2
        i32.load
        local.tee 3
        i32.eqz
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        call_indirect (type 0)
      end
      local.get 2
      i32.load offset=4
      local.tee 3
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 3
      local.get 2
      i32.load offset=8
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 1
      i32.const -1
      i32.eq
      br_if 0 (;@1;)
      local.get 1
      local.get 1
      i32.load offset=4
      local.tee 0
      i32.const -1
      i32.add
      i32.store offset=4
      local.get 0
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 1
      i32.const 320
      i32.const 64
      call $__rust_dealloc
    end)
  (func $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hc628a5a1c0300638E (type 0) (param i32)
    (local i32 i32)
    block  ;; label = @1
      local.get 0
      i32.load
      local.tee 0
      i32.load offset=64
      i32.const -4
      i32.and
      local.tee 1
      i32.load offset=4
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.load
      local.get 2
      i32.const 3
      i32.shl
      i32.const 4
      call $__rust_dealloc
    end
    local.get 1
    i32.const 8
    i32.const 4
    call $__rust_dealloc
    block  ;; label = @1
      local.get 0
      i32.const -1
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      local.get 0
      i32.load offset=4
      local.tee 1
      i32.const -1
      i32.add
      i32.store offset=4
      local.get 1
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 192
      i32.const 64
      call $__rust_dealloc
    end)
  (func $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$3pop17h1ff9952805e7f2a6E (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    i32.const 0
    local.set 2
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.load
        local.tee 3
        i32.load offset=132
        local.tee 4
        local.get 3
        i32.load offset=128
        i32.sub
        local.tee 5
        i32.const 1
        i32.ge_s
        br_if 0 (;@2;)
        br 1 (;@1;)
      end
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 1
            i32.load8_u offset=12
            br_if 0 (;@4;)
            local.get 3
            local.get 3
            i32.load offset=128
            local.tee 6
            i32.const 1
            i32.add
            i32.store offset=128
            local.get 6
            local.get 4
            i32.sub
            i32.const -1
            i32.gt_s
            br_if 1 (;@3;)
            local.get 1
            i32.load offset=8
            local.tee 4
            i32.const 4
            i32.div_s
            local.set 7
            local.get 1
            i32.load offset=4
            local.get 4
            i32.const -1
            i32.add
            local.get 6
            i32.and
            i32.const 3
            i32.shl
            i32.add
            local.tee 2
            i32.load offset=4
            local.set 3
            local.get 2
            i32.load
            local.set 2
            local.get 4
            i32.const 65
            i32.lt_u
            br_if 3 (;@1;)
            local.get 5
            local.get 7
            i32.gt_s
            br_if 3 (;@1;)
            local.get 1
            local.get 4
            i32.const 1
            i32.shr_u
            call $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$6resize17h3d4716eead8d3956E.llvm.6352068949975026865
            br 3 (;@1;)
          end
          local.get 3
          local.get 4
          i32.const -1
          i32.add
          local.tee 5
          i32.store offset=132
          i32.const 0
          local.set 2
          local.get 5
          local.get 1
          i32.load
          local.tee 6
          i32.load offset=128
          local.tee 7
          i32.sub
          local.tee 8
          i32.const 0
          i32.lt_s
          br_if 1 (;@2;)
          local.get 1
          i32.load offset=4
          local.get 1
          i32.load offset=8
          local.tee 9
          i32.const -1
          i32.add
          local.get 5
          i32.and
          i32.const 3
          i32.shl
          i32.add
          local.tee 2
          i32.load offset=4
          local.set 3
          local.get 2
          i32.load
          local.set 2
          block  ;; label = @4
            local.get 5
            local.get 7
            i32.ne
            br_if 0 (;@4;)
            local.get 6
            local.get 4
            local.get 6
            i32.load offset=128
            local.tee 7
            local.get 7
            local.get 5
            i32.eq
            local.tee 5
            select
            i32.store offset=128
            local.get 1
            i32.load
            local.get 4
            i32.store offset=132
            local.get 5
            br_if 3 (;@1;)
            i32.const 0
            local.set 2
            br 3 (;@1;)
          end
          local.get 9
          i32.const 4
          i32.div_s
          local.set 4
          local.get 9
          i32.const 65
          i32.lt_u
          br_if 2 (;@1;)
          local.get 8
          local.get 4
          i32.ge_s
          br_if 2 (;@1;)
          local.get 1
          local.get 9
          i32.const 1
          i32.shr_u
          call $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$6resize17h3d4716eead8d3956E.llvm.6352068949975026865
          br 2 (;@1;)
        end
        local.get 1
        i32.load
        local.get 6
        i32.store offset=128
        br 1 (;@1;)
      end
      local.get 6
      local.get 4
      i32.store offset=132
    end
    local.get 0
    local.get 3
    i32.store offset=4
    local.get 0
    local.get 2
    i32.store)
  (func $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$6resize17h3d4716eead8d3956E.llvm.6352068949975026865 (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 1
    i32.const 3
    i32.shl
    local.set 3
    i32.const 0
    local.set 4
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 1
          i32.const 536870911
          i32.gt_u
          br_if 0 (;@3;)
          local.get 3
          i32.const 2147483644
          i32.gt_u
          br_if 0 (;@3;)
          local.get 0
          i32.load offset=8
          local.set 5
          local.get 0
          i32.load offset=4
          local.set 6
          local.get 0
          i32.load
          local.tee 7
          i32.load offset=128
          local.set 8
          local.get 7
          i32.load offset=132
          local.set 9
          block  ;; label = @4
            block  ;; label = @5
              local.get 3
              br_if 0 (;@5;)
              i32.const 4
              local.set 7
              br 1 (;@4;)
            end
            i32.const 0
            i32.load8_u offset=1058985
            drop
            i32.const 4
            local.set 4
            local.get 3
            i32.const 4
            call $__rust_alloc
            local.tee 7
            i32.eqz
            br_if 1 (;@3;)
          end
          block  ;; label = @4
            local.get 8
            local.get 9
            i32.eq
            br_if 0 (;@4;)
            local.get 1
            i32.const -1
            i32.add
            local.set 4
            local.get 5
            i32.const -1
            i32.add
            local.set 5
            local.get 8
            local.set 3
            block  ;; label = @5
              local.get 9
              local.get 8
              i32.sub
              i32.const 3
              i32.and
              local.tee 10
              i32.eqz
              br_if 0 (;@5;)
              local.get 8
              local.set 3
              loop  ;; label = @6
                local.get 7
                local.get 3
                local.get 4
                i32.and
                i32.const 3
                i32.shl
                i32.add
                local.get 6
                local.get 3
                local.get 5
                i32.and
                i32.const 3
                i32.shl
                i32.add
                i64.load align=4
                i64.store align=4
                local.get 3
                i32.const 1
                i32.add
                local.set 3
                local.get 10
                i32.const -1
                i32.add
                local.tee 10
                br_if 0 (;@6;)
              end
            end
            local.get 8
            local.get 9
            i32.sub
            i32.const -4
            i32.gt_u
            br_if 0 (;@4;)
            loop  ;; label = @5
              local.get 7
              local.get 3
              local.get 4
              i32.and
              i32.const 3
              i32.shl
              i32.add
              local.get 6
              local.get 3
              local.get 5
              i32.and
              i32.const 3
              i32.shl
              i32.add
              i64.load align=4
              i64.store align=4
              local.get 7
              local.get 3
              i32.const 1
              i32.add
              local.tee 10
              local.get 4
              i32.and
              i32.const 3
              i32.shl
              i32.add
              local.get 6
              local.get 10
              local.get 5
              i32.and
              i32.const 3
              i32.shl
              i32.add
              i64.load align=4
              i64.store align=4
              local.get 7
              local.get 3
              i32.const 2
              i32.add
              local.tee 10
              local.get 4
              i32.and
              i32.const 3
              i32.shl
              i32.add
              local.get 6
              local.get 10
              local.get 5
              i32.and
              i32.const 3
              i32.shl
              i32.add
              i64.load align=4
              i64.store align=4
              local.get 7
              local.get 3
              i32.const 3
              i32.add
              local.tee 10
              local.get 4
              i32.and
              i32.const 3
              i32.shl
              i32.add
              local.get 6
              local.get 10
              local.get 5
              i32.and
              i32.const 3
              i32.shl
              i32.add
              i64.load align=4
              i64.store align=4
              local.get 3
              i32.const 4
              i32.add
              local.tee 3
              local.get 9
              i32.ne
              br_if 0 (;@5;)
            end
          end
          i32.const 1059020
          local.set 3
          block  ;; label = @4
            i32.const 0
            i32.load offset=1059016
            br_if 0 (;@4;)
            i32.const 1059016
            i32.const 0
            call $_ZN3std3sys12thread_local6statik20LazyStorage$LT$T$GT$10initialize17ha6e918742de6013dE
            local.set 3
          end
          local.get 2
          local.get 3
          i32.load
          local.tee 3
          i32.store offset=16
          local.get 3
          i32.load offset=1036
          local.tee 6
          i32.const -1
          i32.eq
          br_if 1 (;@2;)
          local.get 3
          local.get 6
          i32.const 1
          i32.add
          i32.store offset=1036
          block  ;; label = @4
            local.get 6
            br_if 0 (;@4;)
            local.get 3
            i32.load offset=4
            i32.load offset=192
            local.set 6
            local.get 3
            local.get 3
            i32.load offset=1044
            local.tee 4
            i32.const 1
            i32.add
            i32.store offset=1044
            local.get 3
            local.get 6
            i32.const 1
            i32.or
            i32.store offset=1088
            local.get 4
            i32.const 127
            i32.and
            br_if 0 (;@4;)
            local.get 3
            i32.load offset=4
            i32.const 64
            i32.add
            local.get 2
            i32.const 16
            i32.add
            call $_ZN15crossbeam_epoch8internal6Global7collect17h7d10bdb777c95448E
          end
          local.get 2
          i32.load offset=16
          local.set 3
          local.get 0
          local.get 1
          i32.store offset=8
          local.get 0
          local.get 7
          i32.store offset=4
          local.get 2
          local.get 3
          i32.store offset=12
          local.get 0
          i32.load
          local.set 6
          i32.const 0
          i32.load8_u offset=1058985
          drop
          i32.const 8
          i32.const 4
          call $__rust_alloc
          local.tee 3
          i32.eqz
          br_if 2 (;@1;)
          local.get 3
          local.get 1
          i32.store offset=4
          local.get 3
          local.get 7
          i32.store
          local.get 6
          i32.load offset=64
          local.set 7
          local.get 6
          local.get 3
          i32.store offset=64
          block  ;; label = @4
            block  ;; label = @5
              local.get 2
              i32.load offset=12
              local.tee 3
              br_if 0 (;@5;)
              block  ;; label = @6
                local.get 7
                i32.const -4
                i32.and
                local.tee 3
                i32.load offset=4
                local.tee 7
                i32.eqz
                br_if 0 (;@6;)
                local.get 3
                i32.load
                local.get 7
                i32.const 3
                i32.shl
                i32.const 4
                call $__rust_dealloc
              end
              local.get 3
              i32.const 8
              i32.const 4
              call $__rust_dealloc
              br 1 (;@4;)
            end
            local.get 2
            local.get 7
            i32.store offset=20
            local.get 2
            i32.const 9
            i32.store offset=16
            local.get 3
            local.get 2
            i32.const 16
            i32.add
            local.get 2
            i32.const 12
            i32.add
            call $_ZN15crossbeam_epoch8internal5Local5defer17h86d466552d8e8645E
          end
          block  ;; label = @4
            local.get 1
            i32.const 128
            i32.lt_u
            br_if 0 (;@4;)
            local.get 2
            i32.const 12
            i32.add
            call $_ZN15crossbeam_epoch5guard5Guard5flush17h32b4d49dc69f4d8aE
          end
          block  ;; label = @4
            local.get 2
            i32.load offset=12
            local.tee 3
            i32.eqz
            br_if 0 (;@4;)
            local.get 3
            local.get 3
            i32.load offset=1036
            local.tee 7
            i32.const -1
            i32.add
            i32.store offset=1036
            local.get 7
            i32.const 1
            i32.ne
            br_if 0 (;@4;)
            local.get 3
            i32.const 0
            i32.store offset=1088
            local.get 3
            i32.load offset=1040
            br_if 0 (;@4;)
            local.get 3
            call $_ZN15crossbeam_epoch8internal5Local8finalize17ha822a731c89268d9E
          end
          local.get 2
          i32.const 32
          i32.add
          global.set $__stack_pointer
          return
        end
        local.get 4
        local.get 3
        i32.const 1048840
        call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
        unreachable
      end
      i32.const 1049708
      call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
      unreachable
    end
    i32.const 4
    i32.const 8
    call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
    unreachable)
  (func $_ZN15crossbeam_deque5deque16Stealer$LT$T$GT$5steal17h92560776915b651eE (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 1
    i32.load
    local.tee 3
    i32.load offset=128
    local.set 4
    block  ;; label = @1
      i32.const 0
      i32.load offset=1059016
      br_if 0 (;@1;)
      i32.const 1059016
      i32.const 0
      call $_ZN3std3sys12thread_local6statik20LazyStorage$LT$T$GT$10initialize17ha6e918742de6013dE
      drop
    end
    i32.const 1059020
    local.set 1
    block  ;; label = @1
      i32.const 0
      i32.load offset=1059016
      br_if 0 (;@1;)
      i32.const 1059016
      i32.const 0
      call $_ZN3std3sys12thread_local6statik20LazyStorage$LT$T$GT$10initialize17ha6e918742de6013dE
      local.set 1
    end
    local.get 2
    local.get 1
    i32.load
    local.tee 1
    i32.store offset=12
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 1
            i32.load offset=1036
            local.tee 5
            i32.const -1
            i32.eq
            br_if 0 (;@4;)
            local.get 1
            local.get 5
            i32.const 1
            i32.add
            i32.store offset=1036
            block  ;; label = @5
              local.get 5
              br_if 0 (;@5;)
              local.get 1
              i32.load offset=4
              i32.load offset=192
              local.set 5
              local.get 1
              local.get 1
              i32.load offset=1044
              local.tee 6
              i32.const 1
              i32.add
              i32.store offset=1044
              local.get 1
              local.get 5
              i32.const 1
              i32.or
              i32.store offset=1088
              local.get 6
              i32.const 127
              i32.and
              br_if 0 (;@5;)
              local.get 1
              i32.load offset=4
              i32.const 64
              i32.add
              local.get 2
              i32.const 12
              i32.add
              call $_ZN15crossbeam_epoch8internal6Global7collect17h7d10bdb777c95448E
            end
            local.get 2
            i32.load offset=12
            local.set 1
            block  ;; label = @5
              local.get 3
              i32.load offset=132
              local.get 4
              i32.sub
              i32.const 1
              i32.ge_s
              br_if 0 (;@5;)
              i32.const 0
              local.set 5
              br 2 (;@3;)
            end
            local.get 3
            i32.load offset=64
            local.tee 6
            i32.const -4
            i32.and
            local.tee 5
            i32.load
            local.get 5
            i32.load offset=4
            i32.const -1
            i32.add
            local.get 4
            i32.and
            i32.const 3
            i32.shl
            i32.add
            local.tee 5
            i32.load offset=4
            local.set 7
            local.get 5
            i32.load
            local.set 8
            i32.const 2
            local.set 5
            local.get 6
            local.get 3
            i32.load offset=64
            i32.ne
            br_if 1 (;@3;)
            local.get 3
            local.get 4
            i32.const 1
            i32.add
            local.get 3
            i32.load offset=128
            local.tee 6
            local.get 6
            local.get 4
            i32.eq
            select
            i32.store offset=128
            local.get 6
            local.get 4
            i32.ne
            br_if 1 (;@3;)
            local.get 0
            local.get 7
            i32.store offset=8
            local.get 0
            local.get 8
            i32.store offset=4
            local.get 0
            i32.const 1
            i32.store
            local.get 1
            i32.eqz
            br_if 3 (;@1;)
            local.get 1
            local.get 1
            i32.load offset=1036
            local.tee 3
            i32.const -1
            i32.add
            i32.store offset=1036
            local.get 3
            i32.const 1
            i32.ne
            br_if 3 (;@1;)
            local.get 1
            i32.const 0
            i32.store offset=1088
            local.get 1
            i32.load offset=1040
            br_if 3 (;@1;)
            br 2 (;@2;)
          end
          i32.const 1049708
          call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
          unreachable
        end
        local.get 0
        local.get 5
        i32.store
        local.get 1
        i32.eqz
        br_if 1 (;@1;)
        local.get 1
        local.get 1
        i32.load offset=1036
        local.tee 3
        i32.const -1
        i32.add
        i32.store offset=1036
        local.get 3
        i32.const 1
        i32.ne
        br_if 1 (;@1;)
        local.get 1
        i32.const 0
        i32.store offset=1088
        local.get 1
        i32.load offset=1040
        br_if 1 (;@1;)
      end
      local.get 1
      call $_ZN15crossbeam_epoch8internal5Local8finalize17ha822a731c89268d9E
    end
    local.get 2
    i32.const 16
    i32.add
    global.set $__stack_pointer)
  (func $__rust_alloc (type 4) (param i32 i32) (result i32)
    (local i32)
    local.get 0
    local.get 1
    call $__rdl_alloc
    local.set 2
    local.get 2
    return)
  (func $__rust_dealloc (type 3) (param i32 i32 i32)
    local.get 0
    local.get 1
    local.get 2
    call $__rdl_dealloc
    return)
  (func $__rust_realloc (type 10) (param i32 i32 i32 i32) (result i32)
    (local i32)
    local.get 0
    local.get 1
    local.get 2
    local.get 3
    call $__rdl_realloc
    local.set 4
    local.get 4
    return)
  (func $__rust_alloc_zeroed (type 4) (param i32 i32) (result i32)
    (local i32)
    local.get 0
    local.get 1
    call $__rdl_alloc_zeroed
    local.set 2
    local.get 2
    return)
  (func $__rust_alloc_error_handler (type 2) (param i32 i32)
    local.get 0
    local.get 1
    call $__rg_oom
    return)
  (func $_ZN4core3ops8function6FnOnce9call_once17hf5860f3535acd078E.llvm.16897747005057573272 (type 5) (param i32) (result i32)
    i32.const 1058996)
  (func $_ZN4core3ptr53drop_in_place$LT$rayon_core..ThreadPoolBuildError$GT$17h765bdb48184b1c4cE.llvm.16897747005057573272 (type 0) (param i32)
    (local i32 i32 i32)
    local.get 0
    i32.load offset=4
    local.set 1
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.load8_u
        local.tee 0
        i32.const 5
        i32.gt_u
        br_if 0 (;@2;)
        local.get 0
        i32.const 3
        i32.ne
        br_if 1 (;@1;)
      end
      local.get 1
      i32.load
      local.set 2
      block  ;; label = @2
        local.get 1
        i32.const 4
        i32.add
        i32.load
        local.tee 0
        i32.load
        local.tee 3
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 3
        call_indirect (type 0)
      end
      block  ;; label = @2
        local.get 0
        i32.load offset=4
        local.tee 3
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 3
        local.get 0
        i32.load offset=8
        call $__rust_dealloc
      end
      local.get 1
      i32.const 12
      i32.const 4
      call $__rust_dealloc
    end)
  (func $_ZN10rayon_core8registry13ThreadBuilder3run17h242dc1e44c4ac4d0E (type 0) (param i32)
    (local i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    local.tee 1
    local.set 2
    local.get 1
    i32.const 256
    i32.sub
    i32.const -64
    i32.and
    local.tee 3
    global.set $__stack_pointer
    local.get 3
    local.get 0
    call $_ZN117_$LT$rayon_core..registry..WorkerThread$u20$as$u20$core..convert..From$LT$rayon_core..registry..ThreadBuilder$GT$$GT$4from17h6e861b9dde3d1c34E
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  i32.const 0
                  i32.load offset=1058996
                  br_if 0 (;@7;)
                  i32.const 0
                  local.get 3
                  i32.store offset=1058996
                  local.get 3
                  i32.load offset=136
                  local.tee 4
                  local.get 3
                  i32.load offset=140
                  local.tee 0
                  i32.load offset=260
                  local.tee 1
                  i32.ge_u
                  br_if 2 (;@5;)
                  local.get 3
                  local.get 0
                  i32.load offset=256
                  local.get 4
                  i32.const 4
                  i32.shl
                  i32.add
                  local.tee 1
                  i32.load8_u offset=12
                  local.tee 5
                  i32.store8 offset=231
                  local.get 1
                  i32.const 1
                  i32.store8 offset=12
                  local.get 5
                  i32.const 1
                  i32.eq
                  br_if 1 (;@6;)
                  local.get 1
                  i32.const 12
                  i32.add
                  i32.const 256
                  i32.store16 align=1
                  block  ;; label = @8
                    local.get 0
                    i32.load offset=216
                    local.tee 1
                    i32.eqz
                    br_if 0 (;@8;)
                    local.get 1
                    local.get 4
                    local.get 0
                    i32.load offset=220
                    i32.load offset=20
                    call_indirect (type 2)
                  end
                  local.get 3
                  i32.load offset=136
                  local.tee 1
                  local.get 3
                  i32.load offset=140
                  local.tee 5
                  i32.load offset=260
                  local.tee 6
                  i32.ge_u
                  br_if 3 (;@4;)
                  block  ;; label = @8
                    local.get 5
                    i32.load offset=256
                    local.get 1
                    i32.const 4
                    i32.shl
                    i32.add
                    local.tee 6
                    i32.load offset=8
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 3
                    local.get 6
                    i32.const 8
                    i32.add
                    call $_ZN10rayon_core8registry12WorkerThread15wait_until_cold17hdc4a872cdf442250E
                  end
                  local.get 1
                  local.get 5
                  i32.load offset=260
                  local.tee 6
                  i32.ge_u
                  br_if 4 (;@3;)
                  local.get 3
                  local.get 5
                  i32.load offset=256
                  local.get 1
                  i32.const 4
                  i32.shl
                  i32.add
                  local.tee 1
                  i32.load8_u offset=14
                  local.tee 5
                  i32.store8 offset=231
                  local.get 1
                  i32.const 1
                  i32.store8 offset=14
                  local.get 5
                  i32.const 1
                  i32.eq
                  br_if 5 (;@2;)
                  local.get 1
                  i32.const 14
                  i32.add
                  i32.const 256
                  i32.store16 align=1
                  block  ;; label = @8
                    local.get 0
                    i32.load offset=224
                    local.tee 1
                    i32.eqz
                    br_if 0 (;@8;)
                    local.get 1
                    local.get 4
                    local.get 0
                    i32.load offset=228
                    i32.load offset=20
                    call_indirect (type 2)
                  end
                  i32.const 0
                  i32.load offset=1058996
                  local.get 3
                  i32.ne
                  br_if 6 (;@1;)
                  i32.const 0
                  i32.const 0
                  i32.store offset=1058996
                  local.get 3
                  i32.load offset=144
                  local.tee 0
                  local.get 0
                  i32.load
                  local.tee 0
                  i32.const -1
                  i32.add
                  i32.store
                  block  ;; label = @8
                    local.get 0
                    i32.const 1
                    i32.ne
                    br_if 0 (;@8;)
                    local.get 3
                    i32.const 144
                    i32.add
                    call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hf1d2b099cab8a99cE
                  end
                  local.get 3
                  i32.load offset=160
                  local.tee 0
                  local.get 0
                  i32.load
                  local.tee 0
                  i32.const -1
                  i32.add
                  i32.store
                  block  ;; label = @8
                    local.get 0
                    i32.const 1
                    i32.ne
                    br_if 0 (;@8;)
                    local.get 3
                    i32.const 160
                    i32.add
                    call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hf1d2b099cab8a99cE
                  end
                  local.get 3
                  i32.load offset=4
                  local.set 1
                  block  ;; label = @8
                    local.get 3
                    i32.load
                    i32.const -2
                    i32.and
                    local.tee 0
                    local.get 3
                    i32.load offset=64
                    i32.const -2
                    i32.and
                    local.tee 5
                    i32.eq
                    br_if 0 (;@8;)
                    loop  ;; label = @9
                      block  ;; label = @10
                        local.get 0
                        i32.const 126
                        i32.and
                        i32.const 126
                        i32.ne
                        br_if 0 (;@10;)
                        local.get 1
                        i32.load
                        local.set 4
                        local.get 1
                        i32.const 760
                        i32.const 4
                        call $__rust_dealloc
                        local.get 4
                        local.set 1
                      end
                      local.get 5
                      local.get 0
                      i32.const 2
                      i32.add
                      local.tee 0
                      i32.ne
                      br_if 0 (;@9;)
                    end
                  end
                  local.get 1
                  i32.const 760
                  i32.const 4
                  call $__rust_dealloc
                  local.get 3
                  i32.load offset=140
                  local.tee 0
                  local.get 0
                  i32.load
                  local.tee 0
                  i32.const -1
                  i32.add
                  i32.store
                  block  ;; label = @8
                    local.get 0
                    i32.const 1
                    i32.ne
                    br_if 0 (;@8;)
                    local.get 3
                    i32.const 140
                    i32.add
                    call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h0960bc771bfafaf1E
                  end
                  local.get 2
                  global.set $__stack_pointer
                  return
                end
                i32.const 1050288
                i32.const 35
                i32.const 1050324
                call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
                unreachable
              end
              local.get 3
              i64.const 0
              i64.store offset=244 align=4
              local.get 3
              i64.const 17179869185
              i64.store offset=236 align=4
              local.get 3
              i32.const 1050580
              i32.store offset=232
              local.get 3
              i32.const 231
              i32.add
              local.get 3
              i32.const 232
              i32.add
              call $_ZN4core9panicking13assert_failed17h9a3e3b4c3cfd5ab0E.llvm.6263538152696972293
              unreachable
            end
            local.get 4
            local.get 1
            i32.const 1050064
            call $_ZN4core9panicking18panic_bounds_check17h6c9fc24fb71a0cb6E
            unreachable
          end
          local.get 1
          local.get 6
          i32.const 1050032
          call $_ZN4core9panicking18panic_bounds_check17h6c9fc24fb71a0cb6E
          unreachable
        end
        local.get 1
        local.get 6
        i32.const 1050048
        call $_ZN4core9panicking18panic_bounds_check17h6c9fc24fb71a0cb6E
        unreachable
      end
      local.get 3
      i64.const 0
      i64.store offset=244 align=4
      local.get 3
      i64.const 17179869185
      i64.store offset=236 align=4
      local.get 3
      i32.const 1050580
      i32.store offset=232
      local.get 3
      i32.const 231
      i32.add
      local.get 3
      i32.const 232
      i32.add
      call $_ZN4core9panicking13assert_failed17h9a3e3b4c3cfd5ab0E.llvm.6263538152696972293
      unreachable
    end
    i32.const 1050120
    i32.const 49
    i32.const 1050272
    call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
    unreachable)
  (func $_ZN117_$LT$rayon_core..registry..WorkerThread$u20$as$u20$core..convert..From$LT$rayon_core..registry..ThreadBuilder$GT$$GT$4from17h6e861b9dde3d1c34E (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i64 i64 i64 i64 i64 i64)
    i32.const 0
    i32.load8_u offset=1058985
    drop
    local.get 1
    i32.load8_u offset=24
    local.set 2
    local.get 1
    i32.load offset=20
    local.set 3
    block  ;; label = @1
      i32.const 760
      i32.const 4
      call $__rust_alloc_zeroed
      local.tee 4
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.const 28
      i32.add
      local.set 5
      local.get 1
      i32.load offset=48
      local.set 6
      local.get 4
      i64.extend_i32_u
      local.set 7
      loop  ;; label = @2
        i32.const 0
        i32.const 0
        i32.load offset=1058988
        local.tee 4
        i32.const 1
        i32.add
        i32.store offset=1058988
        local.get 4
        i64.extend_i32_u
        local.tee 8
        i64.const 8098989879002948979
        i64.xor
        local.tee 9
        i64.const 16
        i64.shl
        i64.const 28773
        i64.or
        local.get 9
        i64.const 7816392313619706465
        i64.add
        i64.xor
        local.tee 10
        i64.const 21
        i64.rotl
        local.get 10
        i64.const -2389207006547353658
        i64.add
        local.tee 10
        i64.xor
        local.tee 11
        i64.const 16
        i64.rotl
        local.get 11
        local.get 9
        i64.const -6481707427168261424
        i64.add
        local.tee 9
        i64.const 32
        i64.rotl
        i64.const 255
        i64.xor
        i64.add
        local.tee 11
        i64.xor
        local.tee 12
        i64.const 21
        i64.rotl
        local.get 12
        local.get 10
        local.get 8
        i64.const 288230376151711744
        i64.or
        i64.xor
        local.get 9
        i64.const -2011800112340241627
        i64.xor
        local.tee 9
        i64.add
        local.tee 8
        i64.const 32
        i64.rotl
        i64.add
        local.tee 10
        i64.xor
        local.tee 12
        i64.const 16
        i64.rotl
        local.get 12
        local.get 8
        local.get 9
        i64.const 13
        i64.shl
        i64.const 7756
        i64.or
        i64.xor
        local.tee 9
        local.get 11
        i64.add
        local.tee 8
        i64.const 32
        i64.rotl
        i64.add
        local.tee 11
        i64.xor
        local.tee 12
        i64.const 21
        i64.rotl
        local.get 12
        local.get 8
        local.get 9
        i64.const 17
        i64.rotl
        i64.xor
        local.tee 9
        local.get 10
        i64.add
        local.tee 8
        i64.const 32
        i64.rotl
        i64.add
        local.tee 10
        i64.xor
        local.tee 12
        i64.const 16
        i64.rotl
        local.get 12
        local.get 9
        i64.const 13
        i64.rotl
        local.get 8
        i64.xor
        local.tee 9
        local.get 11
        i64.add
        local.tee 8
        i64.const 32
        i64.rotl
        i64.add
        local.tee 11
        i64.xor
        i64.const 21
        i64.rotl
        local.get 9
        i64.const 17
        i64.rotl
        local.get 8
        i64.xor
        local.tee 9
        i64.const 13
        i64.rotl
        local.get 9
        local.get 10
        i64.add
        i64.xor
        local.tee 9
        i64.const 17
        i64.rotl
        i64.xor
        local.get 9
        local.get 11
        i64.add
        local.tee 9
        i64.const 32
        i64.rotl
        i64.xor
        local.tee 8
        local.get 9
        i64.eq
        br_if 0 (;@2;)
      end
      local.get 0
      local.get 2
      i32.store8 offset=164
      local.get 0
      local.get 3
      i32.store offset=160
      local.get 0
      local.get 6
      i32.store offset=136
      local.get 0
      local.get 5
      i64.load align=4
      i64.store offset=144 align=4
      local.get 0
      local.get 7
      i64.const 32
      i64.shl
      local.tee 10
      i64.store offset=64
      local.get 0
      local.get 10
      i64.store
      local.get 0
      local.get 1
      i32.load offset=44
      i32.store offset=140
      local.get 0
      local.get 8
      local.get 9
      i64.xor
      i64.store offset=128
      local.get 0
      i32.const 152
      i32.add
      local.get 5
      i32.const 8
      i32.add
      i64.load align=4
      i64.store align=4
      block  ;; label = @2
        local.get 1
        i32.load offset=8
        local.tee 4
        i32.const -2147483648
        i32.eq
        br_if 0 (;@2;)
        local.get 4
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.load offset=12
        local.get 4
        i32.const 1
        call $__rust_dealloc
      end
      return
    end
    i32.const 4
    i32.const 760
    call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
    unreachable)
  (func $_ZN10rayon_core8registry12WorkerThread15wait_until_cold17hdc4a872cdf442250E (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i64 i64 i32)
    global.get $__stack_pointer
    i32.const 96
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      local.get 1
      i32.load
      i32.const 3
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      i32.const 160
      i32.add
      local.set 3
      local.get 0
      i32.const 144
      i32.add
      local.set 4
      local.get 2
      i32.const 76
      i32.add
      local.set 5
      loop  ;; label = @2
        local.get 2
        i32.const 24
        i32.add
        local.get 4
        call $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$3pop17h1ca15d87b82f6609E
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                local.get 2
                i32.load offset=24
                local.tee 6
                i32.eqz
                br_if 0 (;@6;)
                local.get 2
                i32.load offset=28
                local.set 7
                br 1 (;@5;)
              end
              loop  ;; label = @6
                local.get 2
                i32.const 52
                i32.add
                local.get 3
                call $_ZN15crossbeam_deque5deque16Stealer$LT$T$GT$5steal17hb6210202028f7ff8E
                local.get 2
                i32.load offset=52
                local.tee 6
                i32.const 2
                i32.eq
                br_if 0 (;@6;)
              end
              block  ;; label = @6
                local.get 6
                br_table 2 (;@4;) 0 (;@6;) 2 (;@4;)
              end
              local.get 2
              i32.load offset=60
              local.set 7
              local.get 2
              i32.load offset=56
              local.set 6
            end
            local.get 7
            local.get 6
            call_indirect (type 0)
            br 1 (;@3;)
          end
          local.get 0
          i32.load offset=136
          local.set 6
          local.get 0
          i32.load offset=140
          local.tee 7
          local.get 7
          i32.load offset=248
          i32.const 256
          i32.add
          i32.store offset=248
          local.get 2
          i64.const -4294967296
          i64.store offset=40 align=4
          local.get 2
          local.get 6
          i32.store offset=36
          block  ;; label = @4
            local.get 1
            i32.load
            i32.const 3
            i32.eq
            br_if 0 (;@4;)
            block  ;; label = @5
              loop  ;; label = @6
                local.get 2
                i32.const 16
                i32.add
                local.get 4
                call $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$3pop17h1ca15d87b82f6609E
                block  ;; label = @7
                  local.get 2
                  i32.load offset=16
                  local.tee 7
                  i32.eqz
                  br_if 0 (;@7;)
                  local.get 2
                  i32.load offset=20
                  local.set 8
                  br 2 (;@5;)
                end
                loop  ;; label = @7
                  local.get 2
                  i32.const 52
                  i32.add
                  local.get 3
                  call $_ZN15crossbeam_deque5deque16Stealer$LT$T$GT$5steal17hb6210202028f7ff8E
                  local.get 2
                  i32.load offset=52
                  local.tee 6
                  i32.const 2
                  i32.eq
                  br_if 0 (;@7;)
                end
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      block  ;; label = @10
                        block  ;; label = @11
                          block  ;; label = @12
                            local.get 6
                            br_table 0 (;@12;) 1 (;@11;) 0 (;@12;)
                          end
                          block  ;; label = @12
                            local.get 0
                            i32.load offset=140
                            local.tee 7
                            i32.load offset=260
                            local.tee 6
                            i32.const 2
                            i32.lt_u
                            br_if 0 (;@12;)
                            local.get 7
                            i32.load offset=256
                            local.set 8
                            local.get 6
                            i64.extend_i32_u
                            local.set 9
                            loop  ;; label = @13
                              local.get 0
                              local.get 0
                              i64.load offset=128
                              local.tee 10
                              i64.const 12
                              i64.shr_u
                              local.get 10
                              i64.xor
                              local.tee 10
                              i64.const 25
                              i64.shl
                              local.get 10
                              i64.xor
                              local.tee 10
                              i64.const 27
                              i64.shr_u
                              local.get 10
                              i64.xor
                              local.tee 10
                              i64.store offset=128
                              local.get 2
                              i32.const 0
                              i32.store8 offset=51
                              local.get 2
                              local.get 0
                              i32.store offset=76
                              local.get 2
                              i64.const 1
                              i64.store offset=64 align=4
                              local.get 2
                              local.get 6
                              i32.store offset=60
                              local.get 2
                              i32.const 1
                              i32.store offset=52
                              local.get 2
                              local.get 10
                              i64.const 2685821657736338717
                              i64.mul
                              local.get 9
                              i64.rem_u
                              i32.wrap_i64
                              local.tee 7
                              i32.store offset=72
                              local.get 2
                              local.get 7
                              i32.store offset=56
                              local.get 2
                              local.get 6
                              i32.store offset=88
                              local.get 2
                              local.get 8
                              i32.store offset=84
                              local.get 2
                              local.get 5
                              i32.store offset=80
                              local.get 2
                              local.get 2
                              i32.const 51
                              i32.add
                              i32.store offset=92
                              local.get 2
                              i32.const 8
                              i32.add
                              local.get 2
                              i32.const 52
                              i32.add
                              local.get 2
                              i32.const 80
                              i32.add
                              call $_ZN106_$LT$core..iter..adapters..chain..Chain$LT$A$C$B$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$8try_fold17h8f59e602544a6002E
                              local.get 2
                              i32.load offset=8
                              local.tee 7
                              br_if 4 (;@9;)
                              local.get 2
                              i32.load8_u offset=51
                              br_if 0 (;@13;)
                            end
                            local.get 0
                            i32.load offset=140
                            local.set 7
                          end
                          local.get 7
                          i32.const 64
                          i32.add
                          local.set 6
                          loop  ;; label = @12
                            local.get 2
                            i32.const 52
                            i32.add
                            local.get 6
                            call $_ZN15crossbeam_deque5deque17Injector$LT$T$GT$5steal17h460d132b966242adE
                            local.get 2
                            i32.load offset=52
                            local.tee 7
                            i32.const 2
                            i32.eq
                            br_if 0 (;@12;)
                          end
                          local.get 7
                          br_table 1 (;@10;) 0 (;@11;) 1 (;@10;)
                        end
                        local.get 2
                        i32.load offset=60
                        local.set 8
                        local.get 2
                        i32.load offset=56
                        local.set 7
                        br 5 (;@5;)
                      end
                      local.get 2
                      i32.load offset=40
                      local.tee 6
                      i32.const 32
                      i32.lt_u
                      br_if 1 (;@8;)
                      local.get 0
                      i32.load offset=140
                      local.set 7
                      block  ;; label = @10
                        local.get 6
                        i32.const 32
                        i32.ne
                        br_if 0 (;@10;)
                        block  ;; label = @11
                          loop  ;; label = @12
                            block  ;; label = @13
                              local.get 7
                              i32.load offset=248
                              local.tee 6
                              i32.const 65536
                              i32.and
                              br_if 0 (;@13;)
                              local.get 6
                              local.set 11
                              br 2 (;@11;)
                            end
                            local.get 7
                            local.get 6
                            i32.const 65536
                            i32.add
                            local.tee 11
                            local.get 7
                            i32.load offset=248
                            local.tee 8
                            local.get 8
                            local.get 6
                            i32.eq
                            select
                            i32.store offset=248
                            local.get 8
                            local.get 6
                            i32.ne
                            br_if 0 (;@12;)
                          end
                        end
                        local.get 2
                        i32.const 33
                        i32.store offset=40
                        local.get 2
                        local.get 11
                        i32.const 16
                        i32.shr_u
                        i32.store offset=44
                        br 3 (;@7;)
                      end
                      local.get 7
                      i32.const 236
                      i32.add
                      local.get 2
                      i32.const 36
                      i32.add
                      local.get 1
                      local.get 0
                      call $_ZN10rayon_core5sleep5Sleep5sleep17h61ecc6b84acce2aaE
                      br 2 (;@7;)
                    end
                    local.get 2
                    i32.load offset=12
                    local.set 8
                    br 3 (;@5;)
                  end
                  local.get 2
                  local.get 6
                  i32.const 1
                  i32.add
                  i32.store offset=40
                end
                local.get 1
                i32.load
                i32.const 3
                i32.ne
                br_if 0 (;@6;)
                br 2 (;@4;)
              end
            end
            local.get 0
            i32.load offset=140
            local.tee 6
            local.get 6
            i32.load offset=248
            local.tee 11
            i32.const -256
            i32.add
            i32.store offset=248
            local.get 6
            i32.const 236
            i32.add
            local.get 11
            i32.const 255
            i32.and
            local.tee 6
            i32.const 2
            local.get 6
            i32.const 2
            i32.lt_u
            select
            call $_ZN10rayon_core5sleep5Sleep16wake_any_threads17h17244bbca7ed393dE
            local.get 8
            local.get 7
            call_indirect (type 0)
            br 1 (;@3;)
          end
          local.get 0
          i32.load offset=140
          local.tee 3
          local.get 3
          i32.load offset=248
          local.tee 6
          i32.const -256
          i32.add
          i32.store offset=248
          local.get 3
          i32.const 236
          i32.add
          local.get 6
          i32.const 255
          i32.and
          local.tee 3
          i32.const 2
          local.get 3
          i32.const 2
          i32.lt_u
          select
          call $_ZN10rayon_core5sleep5Sleep16wake_any_threads17h17244bbca7ed393dE
          br 2 (;@1;)
        end
        local.get 1
        i32.load
        i32.const 3
        i32.ne
        br_if 0 (;@2;)
      end
    end
    local.get 2
    i32.const 96
    i32.add
    global.set $__stack_pointer)
  (func $_ZN88_$LT$rayon_core..registry..DefaultSpawn$u20$as$u20$rayon_core..registry..ThreadSpawn$GT$5spawn17hde45ef0851463ac6E (type 3) (param i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32 i64 i32)
    global.get $__stack_pointer
    i32.const 96
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    i32.const -2147483648
    local.set 4
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 2
          i32.load offset=8
          i32.const -2147483648
          i32.ne
          br_if 0 (;@3;)
          i32.const 0
          local.set 5
          i32.const 0
          local.set 6
          br 1 (;@2;)
        end
        local.get 2
        i32.load offset=12
        local.set 4
        local.get 2
        i32.load offset=16
        local.set 7
        i32.const 0
        local.set 8
        local.get 3
        i32.const 0
        i32.store8 offset=52
        local.get 3
        i32.const -2147483648
        i32.store offset=40
        local.get 3
        i32.const 0
        i32.store offset=32
        local.get 7
        i32.const 0
        i32.lt_s
        br_if 1 (;@1;)
        block  ;; label = @3
          block  ;; label = @4
            local.get 7
            br_if 0 (;@4;)
            i32.const 1
            local.set 5
            br 1 (;@3;)
          end
          i32.const 0
          i32.load8_u offset=1058985
          drop
          i32.const 1
          local.set 8
          local.get 7
          i32.const 1
          call $__rust_alloc
          local.tee 5
          i32.eqz
          br_if 2 (;@1;)
        end
        local.get 5
        local.get 4
        local.get 7
        call $memcpy
        local.set 4
        local.get 3
        local.get 7
        i32.store offset=92
        local.get 3
        local.get 4
        i32.store offset=88
        local.get 3
        local.get 7
        i32.store offset=84
        local.get 3
        i32.const 8
        i32.add
        local.get 3
        i32.const 32
        i32.add
        local.get 3
        i32.const 84
        i32.add
        call $_ZN3std6thread7Builder4name17h4203e57084d3f707E
        local.get 3
        i32.const 6
        i32.add
        local.get 3
        i32.const 31
        i32.add
        i32.load8_u
        i32.store8
        local.get 3
        local.get 3
        i32.load16_u offset=29 align=1
        i32.store16 offset=4
        local.get 3
        i32.load offset=8
        local.set 6
        local.get 3
        i32.load offset=12
        local.set 8
        local.get 3
        i32.load offset=16
        local.set 4
        local.get 3
        i64.load offset=20 align=4
        local.set 9
        local.get 3
        i32.load8_u offset=28
        local.set 5
      end
      local.get 2
      i32.load offset=4
      local.set 10
      local.get 2
      i32.load
      local.set 7
      local.get 3
      i32.const 31
      i32.add
      local.get 3
      i32.const 6
      i32.add
      i32.load8_u
      i32.store8
      local.get 3
      local.get 5
      i32.store8 offset=28
      local.get 3
      local.get 9
      i64.store offset=20 align=4
      local.get 3
      local.get 4
      i32.store offset=16
      local.get 3
      local.get 10
      local.get 8
      local.get 7
      select
      i32.store offset=12
      local.get 3
      i32.const 1
      local.get 6
      local.get 7
      select
      i32.store offset=8
      local.get 3
      local.get 3
      i32.load16_u offset=4
      i32.store16 offset=29 align=1
      local.get 3
      i32.const 32
      i32.add
      i32.const 48
      i32.add
      local.get 2
      i32.const 48
      i32.add
      i32.load
      i32.store
      local.get 3
      i32.const 32
      i32.add
      i32.const 40
      i32.add
      local.get 2
      i32.const 40
      i32.add
      i64.load align=4
      i64.store
      local.get 3
      i32.const 32
      i32.add
      i32.const 32
      i32.add
      local.get 2
      i32.const 32
      i32.add
      i64.load align=4
      i64.store
      local.get 3
      i32.const 32
      i32.add
      i32.const 24
      i32.add
      local.get 2
      i32.const 24
      i32.add
      i64.load align=4
      i64.store
      local.get 3
      i32.const 32
      i32.add
      i32.const 16
      i32.add
      local.get 2
      i32.const 16
      i32.add
      i64.load align=4
      i64.store
      local.get 3
      i32.const 32
      i32.add
      i32.const 8
      i32.add
      local.get 2
      i32.const 8
      i32.add
      i64.load align=4
      i64.store
      local.get 3
      local.get 2
      i64.load align=4
      i64.store offset=32
      local.get 3
      i32.const 84
      i32.add
      local.get 3
      i32.const 8
      i32.add
      local.get 3
      i32.const 32
      i32.add
      i32.const 0
      call $_ZN3std6thread7Builder16spawn_unchecked_17h8de3712b4d76d82fE
      local.get 0
      local.get 3
      i64.load offset=88 align=4
      i64.store align=4
      local.get 3
      i32.const 96
      i32.add
      global.set $__stack_pointer
      return
    end
    local.get 8
    local.get 7
    i32.const 1049800
    call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
    unreachable)
  (func $_ZN10rayon_core8registry15global_registry17hf6a3fbcd34bc87c8E (type 11) (result i32)
    (local i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 0
    global.set $__stack_pointer
    local.get 0
    i64.const 4
    i64.store offset=24
    i32.const 0
    local.set 1
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          i32.const 0
          i32.load8_u offset=1059004
          i32.const 3
          i32.ne
          br_if 0 (;@3;)
          local.get 0
          i32.const 14
          i32.add
          local.get 0
          i32.load8_u offset=27
          i32.store8
          local.get 0
          local.get 0
          i32.load16_u offset=25 align=1
          i32.store16 offset=12
          i32.const 4
          local.set 2
          br 1 (;@2;)
        end
        local.get 0
        local.get 0
        i32.const 24
        i32.add
        i32.store offset=16
        local.get 0
        local.get 0
        i32.const 16
        i32.add
        i32.store offset=20
        i32.const 1059004
        i32.const 0
        local.get 0
        i32.const 20
        i32.add
        i32.const 1049920
        call $_ZN3std3sys4sync4once10no_threads4Once4call17h8c7d0a8c72d63a8fE
        local.get 0
        i32.const 14
        i32.add
        local.get 0
        i32.load8_u offset=27
        i32.store8
        local.get 0
        local.get 0
        i32.load16_u offset=25 align=1
        i32.store16 offset=12
        local.get 0
        i32.load offset=28
        local.tee 1
        local.set 3
        local.get 0
        i32.load8_u offset=24
        local.tee 2
        i32.const 6
        i32.eq
        br_if 1 (;@1;)
      end
      block  ;; label = @2
        i32.const 0
        i32.load offset=1059000
        br_if 0 (;@2;)
        local.get 0
        local.get 0
        i32.load16_u offset=12
        i32.store16 offset=25 align=1
        local.get 0
        local.get 1
        i32.store offset=28
        local.get 0
        local.get 2
        i32.store8 offset=24
        local.get 0
        local.get 0
        i32.const 14
        i32.add
        i32.load8_u
        i32.store8 offset=27
        i32.const 1049952
        i32.const 48
        local.get 0
        i32.const 24
        i32.add
        i32.const 1049936
        i32.const 1050000
        call $_ZN4core6result13unwrap_failed17h89eac97f11bebdf4E
        unreachable
      end
      block  ;; label = @2
        local.get 2
        i32.const 6
        i32.ge_u
        br_if 0 (;@2;)
        i32.const 1059000
        local.set 3
        i32.const 55
        local.get 2
        i32.shr_u
        i32.const 1
        i32.and
        br_if 1 (;@1;)
      end
      local.get 1
      i32.load
      local.set 3
      block  ;; label = @2
        local.get 1
        i32.const 4
        i32.add
        i32.load
        local.tee 2
        i32.load
        local.tee 4
        i32.eqz
        br_if 0 (;@2;)
        local.get 3
        local.get 4
        call_indirect (type 0)
      end
      block  ;; label = @2
        local.get 2
        i32.load offset=4
        local.tee 4
        i32.eqz
        br_if 0 (;@2;)
        local.get 3
        local.get 4
        local.get 2
        i32.load offset=8
        call $__rust_dealloc
      end
      local.get 1
      i32.const 12
      i32.const 4
      call $__rust_dealloc
      i32.const 1059000
      local.set 3
    end
    local.get 0
    i32.const 32
    i32.add
    global.set $__stack_pointer
    local.get 3)
  (func $_ZN10rayon_core8registry23default_global_registry17he1e6784285ac3562E (type 0) (param i32)
    (local i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 112
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 1
    i32.const 0
    i32.store16 offset=108
    local.get 1
    i32.const 0
    i32.store offset=100
    local.get 1
    i32.const 0
    i32.store offset=92
    local.get 1
    i32.const 0
    i32.store offset=84
    local.get 1
    i64.const 0
    i64.store offset=72 align=4
    local.get 1
    i32.const 0
    i32.store offset=64
    local.get 1
    i32.const 8
    i32.add
    local.get 1
    i32.const 64
    i32.add
    call $_ZN10rayon_core8registry8Registry3new17hb2909e93c8e26fa5E
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.load8_u offset=8
        local.tee 2
        i32.const 6
        i32.eq
        br_if 0 (;@2;)
        local.get 2
        i32.const 6
        i32.and
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                local.get 2
                br_table 4 (;@2;) 0 (;@6;) 1 (;@5;) 2 (;@4;) 4 (;@2;)
              end
              local.get 1
              i32.load8_u offset=9
              local.set 3
              br 2 (;@3;)
            end
            local.get 1
            i32.load offset=12
            i32.load8_u offset=8
            local.set 3
            br 1 (;@3;)
          end
          local.get 1
          i32.load offset=12
          i32.load8_u offset=8
          local.set 3
        end
        local.get 3
        i32.const 255
        i32.and
        i32.const 36
        i32.ne
        br_if 0 (;@2;)
        i32.const 0
        i32.load offset=1058996
        br_if 0 (;@2;)
        local.get 1
        i32.const 1
        i32.store16 offset=60
        local.get 1
        i32.const 0
        i32.store offset=52
        local.get 1
        i32.const 0
        i32.store offset=44
        local.get 1
        i32.const 0
        i32.store offset=36
        local.get 1
        i64.const 1
        i64.store offset=24 align=4
        local.get 1
        i32.const 0
        i32.store offset=16
        local.get 1
        i32.const 64
        i32.add
        local.get 1
        i32.const 16
        i32.add
        call $_ZN10rayon_core8registry8Registry3new17hb2909e93c8e26fa5E
        block  ;; label = @3
          local.get 1
          i32.load8_u offset=64
          local.tee 3
          i32.const 6
          i32.ne
          br_if 0 (;@3;)
          local.get 0
          local.get 1
          i64.load offset=64
          i64.store align=4
          local.get 2
          i32.const -1
          i32.add
          i32.const 2
          i32.lt_u
          br_if 2 (;@1;)
          local.get 1
          i32.load offset=12
          local.tee 0
          i32.load
          local.set 3
          block  ;; label = @4
            local.get 0
            i32.const 4
            i32.add
            i32.load
            local.tee 2
            i32.load
            local.tee 4
            i32.eqz
            br_if 0 (;@4;)
            local.get 3
            local.get 4
            call_indirect (type 0)
          end
          block  ;; label = @4
            local.get 2
            i32.load offset=4
            local.tee 4
            i32.eqz
            br_if 0 (;@4;)
            local.get 3
            local.get 4
            local.get 2
            i32.load offset=8
            call $__rust_dealloc
          end
          local.get 0
          i32.const 12
          i32.const 4
          call $__rust_dealloc
          br 2 (;@1;)
        end
        local.get 1
        i32.load offset=68
        local.set 2
        block  ;; label = @3
          local.get 3
          i32.const 5
          i32.gt_u
          br_if 0 (;@3;)
          local.get 3
          i32.const 3
          i32.ne
          br_if 1 (;@2;)
        end
        local.get 2
        i32.load
        local.set 4
        block  ;; label = @3
          local.get 2
          i32.const 4
          i32.add
          i32.load
          local.tee 3
          i32.load
          local.tee 5
          i32.eqz
          br_if 0 (;@3;)
          local.get 4
          local.get 5
          call_indirect (type 0)
        end
        block  ;; label = @3
          local.get 3
          i32.load offset=4
          local.tee 5
          i32.eqz
          br_if 0 (;@3;)
          local.get 4
          local.get 5
          local.get 3
          i32.load offset=8
          call $__rust_dealloc
        end
        local.get 2
        i32.const 12
        i32.const 4
        call $__rust_dealloc
      end
      local.get 0
      local.get 1
      i64.load offset=8
      i64.store align=4
    end
    local.get 1
    i32.const 112
    i32.add
    global.set $__stack_pointer)
  (func $_ZN10rayon_core8registry8Registry3new17hb2909e93c8e26fa5E (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i64 i32 i32 i32 i32 i32 i32 i64 i32 i32 i32)
    global.get $__stack_pointer
    local.tee 2
    local.set 3
    local.get 2
    i32.const 384
    i32.sub
    i32.const -64
    i32.and
    local.tee 4
    global.set $__stack_pointer
    local.get 1
    i32.load offset=8
    local.set 5
    local.get 4
    local.get 1
    i32.load8_u offset=45
    i32.store8 offset=39
    local.get 4
    local.get 5
    i32.const 1
    local.get 5
    i32.const 1
    i32.gt_u
    select
    local.tee 2
    i32.const 255
    local.get 2
    i32.const 255
    i32.lt_u
    select
    local.tee 6
    i32.store offset=336
    i32.const 0
    local.set 7
    local.get 4
    i32.const 0
    i32.store offset=332
    local.get 4
    local.get 4
    i32.const 39
    i32.add
    i32.store offset=328
    local.get 4
    i32.const 128
    i32.add
    local.get 4
    i32.const 328
    i32.add
    call $_ZN4core4iter6traits8iterator8Iterator5unzip17h1f17632128fc2d60E
    local.get 4
    i32.load offset=128
    local.set 8
    local.get 4
    i32.load offset=132
    local.set 9
    local.get 4
    i32.load offset=136
    local.set 10
    local.get 4
    i32.load offset=140
    local.set 11
    local.get 4
    i32.load offset=144
    local.set 12
    local.get 4
    i32.load offset=148
    local.set 2
    local.get 4
    i32.const 128
    i32.add
    i32.const 0
    local.get 6
    call $_ZN4core4iter6traits8iterator8Iterator5unzip17he9948bad0b92d0b4E
    local.get 4
    i32.const 328
    i32.add
    i32.const 8
    i32.add
    local.get 4
    i32.const 128
    i32.add
    i32.const 8
    i32.add
    i32.load
    i32.store
    local.get 4
    local.get 4
    i64.load offset=128 align=4
    i64.store offset=328
    i32.const 4
    local.set 13
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      block  ;; label = @10
                        local.get 2
                        i32.const 4
                        i32.shl
                        local.tee 14
                        i32.const 2147483644
                        i32.gt_u
                        br_if 0 (;@10;)
                        local.get 4
                        i32.load offset=140
                        local.set 15
                        local.get 4
                        i32.load offset=144
                        local.set 16
                        local.get 4
                        i32.load offset=148
                        local.set 17
                        i32.const 0
                        local.set 18
                        i32.const 0
                        local.set 19
                        block  ;; label = @11
                          local.get 14
                          i32.eqz
                          br_if 0 (;@11;)
                          i32.const 0
                          i32.load8_u offset=1058985
                          drop
                          i32.const 4
                          local.set 7
                          local.get 14
                          i32.const 4
                          call $__rust_alloc
                          local.tee 13
                          i32.eqz
                          br_if 1 (;@10;)
                          local.get 2
                          local.set 19
                        end
                        block  ;; label = @11
                          local.get 2
                          i32.eqz
                          br_if 0 (;@11;)
                          local.get 2
                          i32.const -1
                          i32.add
                          i32.const 536870911
                          i32.and
                          local.tee 2
                          i32.const 1
                          i32.add
                          local.tee 14
                          i32.const 3
                          i32.and
                          local.set 7
                          block  ;; label = @12
                            block  ;; label = @13
                              local.get 2
                              i32.const 3
                              i32.ge_u
                              br_if 0 (;@13;)
                              i32.const 0
                              local.set 18
                              local.get 12
                              local.set 2
                              br 1 (;@12;)
                            end
                            local.get 14
                            i32.const 1073741820
                            i32.and
                            local.set 20
                            i32.const 0
                            local.set 18
                            local.get 13
                            local.set 14
                            local.get 12
                            local.set 2
                            loop  ;; label = @13
                              local.get 2
                              i32.load
                              local.set 21
                              local.get 2
                              i32.load8_u offset=4
                              local.set 22
                              local.get 14
                              i32.const 8
                              i32.add
                              i64.const 0
                              i64.store align=4
                              local.get 14
                              i32.const 4
                              i32.add
                              local.get 22
                              i32.store8
                              local.get 14
                              local.get 21
                              i32.store
                              local.get 2
                              i32.const 8
                              i32.add
                              i32.load
                              local.set 21
                              local.get 2
                              i32.load8_u offset=12
                              local.set 22
                              local.get 14
                              i32.const 24
                              i32.add
                              i64.const 0
                              i64.store align=4
                              local.get 14
                              i32.const 20
                              i32.add
                              local.get 22
                              i32.store8
                              local.get 14
                              i32.const 16
                              i32.add
                              local.get 21
                              i32.store
                              local.get 2
                              i32.const 16
                              i32.add
                              i32.load
                              local.set 21
                              local.get 2
                              i32.load8_u offset=20
                              local.set 22
                              local.get 14
                              i32.const 40
                              i32.add
                              i64.const 0
                              i64.store align=4
                              local.get 14
                              i32.const 36
                              i32.add
                              local.get 22
                              i32.store8
                              local.get 14
                              i32.const 32
                              i32.add
                              local.get 21
                              i32.store
                              local.get 2
                              i32.const 24
                              i32.add
                              i32.load
                              local.set 21
                              local.get 2
                              i32.load8_u offset=28
                              local.set 22
                              local.get 14
                              i32.const 56
                              i32.add
                              i64.const 0
                              i64.store align=4
                              local.get 14
                              i32.const 52
                              i32.add
                              local.get 22
                              i32.store8
                              local.get 14
                              i32.const 48
                              i32.add
                              local.get 21
                              i32.store
                              local.get 2
                              i32.const 32
                              i32.add
                              local.set 2
                              local.get 14
                              i32.const 64
                              i32.add
                              local.set 14
                              local.get 20
                              local.get 18
                              i32.const 4
                              i32.add
                              local.tee 18
                              i32.ne
                              br_if 0 (;@13;)
                            end
                          end
                          local.get 7
                          i32.eqz
                          br_if 0 (;@11;)
                          local.get 18
                          local.get 7
                          i32.add
                          local.set 22
                          local.get 13
                          local.get 18
                          i32.const 4
                          i32.shl
                          i32.add
                          local.set 14
                          loop  ;; label = @12
                            local.get 2
                            i32.load
                            local.set 21
                            local.get 2
                            i32.load8_u offset=4
                            local.set 18
                            local.get 14
                            i32.const 8
                            i32.add
                            i64.const 0
                            i64.store align=4
                            local.get 14
                            i32.const 4
                            i32.add
                            local.get 18
                            i32.store8
                            local.get 14
                            local.get 21
                            i32.store
                            local.get 2
                            i32.const 8
                            i32.add
                            local.set 2
                            local.get 14
                            i32.const 16
                            i32.add
                            local.set 14
                            local.get 7
                            i32.const -1
                            i32.add
                            local.tee 7
                            br_if 0 (;@12;)
                          end
                          local.get 22
                          local.set 18
                        end
                        block  ;; label = @11
                          local.get 11
                          i32.eqz
                          br_if 0 (;@11;)
                          local.get 12
                          local.get 11
                          i32.const 3
                          i32.shl
                          i32.const 4
                          call $__rust_dealloc
                        end
                        i32.const 0
                        local.set 21
                        i32.const 0
                        i32.load8_u offset=1058985
                        drop
                        local.get 6
                        i32.const 6
                        i32.shl
                        local.tee 2
                        i32.const 64
                        call $__rust_alloc
                        local.tee 22
                        i32.eqz
                        br_if 1 (;@9;)
                        local.get 6
                        i32.const 3
                        i32.and
                        local.set 7
                        block  ;; label = @11
                          local.get 5
                          i32.const 4
                          i32.lt_u
                          br_if 0 (;@11;)
                          local.get 6
                          i32.const 6
                          i32.shl
                          i32.const -256
                          i32.and
                          local.set 20
                          i32.const 0
                          local.set 14
                          i32.const 0
                          local.set 21
                          loop  ;; label = @12
                            local.get 22
                            local.get 14
                            i32.add
                            local.tee 2
                            i32.const 0
                            i32.store16
                            local.get 2
                            i32.const 192
                            i32.add
                            i32.const 0
                            i32.store16
                            local.get 2
                            i32.const 128
                            i32.add
                            i32.const 0
                            i32.store16
                            local.get 2
                            i32.const 64
                            i32.add
                            i32.const 0
                            i32.store16
                            local.get 21
                            i32.const 4
                            i32.add
                            local.set 21
                            local.get 20
                            local.get 14
                            i32.const 256
                            i32.add
                            local.tee 14
                            i32.ne
                            br_if 0 (;@12;)
                          end
                        end
                        block  ;; label = @11
                          local.get 7
                          i32.eqz
                          br_if 0 (;@11;)
                          local.get 22
                          local.get 21
                          i32.const 6
                          i32.shl
                          i32.add
                          local.set 2
                          loop  ;; label = @12
                            local.get 2
                            i32.const 0
                            i32.store16
                            local.get 2
                            i32.const 64
                            i32.add
                            local.set 2
                            local.get 7
                            i32.const -1
                            i32.add
                            local.tee 7
                            br_if 0 (;@12;)
                          end
                        end
                        i32.const 0
                        i32.load8_u offset=1058985
                        drop
                        i32.const 760
                        i32.const 4
                        call $__rust_alloc_zeroed
                        local.tee 2
                        i32.eqz
                        br_if 2 (;@8;)
                        local.get 4
                        i32.const 56
                        i32.add
                        local.get 4
                        i32.const 336
                        i32.add
                        i32.load
                        i32.store align=1
                        local.get 4
                        local.get 4
                        i64.load offset=328
                        i64.store offset=48 align=1
                        local.get 1
                        i32.load offset=12
                        local.set 14
                        local.get 1
                        i32.const 0
                        i32.store offset=12
                        local.get 1
                        i32.load offset=28
                        local.set 21
                        local.get 1
                        i32.const 0
                        i32.store offset=28
                        local.get 1
                        i32.load offset=36
                        local.set 20
                        local.get 1
                        i32.const 0
                        i32.store offset=36
                        i32.const 0
                        i32.load8_u offset=1058985
                        drop
                        local.get 1
                        i32.load offset=40
                        local.set 5
                        local.get 1
                        i32.load offset=32
                        local.set 11
                        local.get 1
                        i32.load offset=16
                        local.set 12
                        i32.const 320
                        i32.const 64
                        call $__rust_alloc
                        local.tee 7
                        i32.eqz
                        br_if 3 (;@7;)
                        local.get 7
                        i32.const 0
                        i32.store8 offset=192
                        local.get 7
                        i64.const 4294967297
                        i64.store
                        local.get 7
                        local.get 4
                        i64.load offset=45 align=1
                        i64.store offset=193 align=1
                        local.get 7
                        local.get 18
                        i32.store offset=260
                        local.get 7
                        local.get 13
                        i32.store offset=256
                        local.get 7
                        local.get 19
                        i32.store offset=252
                        local.get 7
                        i32.const 0
                        i32.store offset=248
                        local.get 7
                        local.get 6
                        i32.store offset=244
                        local.get 7
                        local.get 22
                        i32.store offset=240
                        local.get 7
                        local.get 6
                        i32.store offset=236
                        local.get 7
                        i32.const 1
                        i32.store offset=232
                        local.get 7
                        local.get 5
                        i32.store offset=228
                        local.get 7
                        local.get 20
                        i32.store offset=224
                        local.get 7
                        local.get 11
                        i32.store offset=220
                        local.get 7
                        local.get 21
                        i32.store offset=216
                        local.get 7
                        local.get 12
                        i32.store offset=212
                        local.get 7
                        local.get 14
                        i32.store offset=208
                        local.get 7
                        local.get 2
                        i64.extend_i32_u
                        i64.const 32
                        i64.shl
                        local.tee 23
                        i64.store offset=128
                        local.get 7
                        local.get 23
                        i64.store offset=64
                        local.get 7
                        i32.const 200
                        i32.add
                        local.get 4
                        i32.const 52
                        i32.add
                        i64.load align=1
                        i64.store align=1
                        local.get 4
                        local.get 7
                        i32.store offset=40
                        local.get 16
                        local.get 17
                        i32.const 3
                        i32.shl
                        i32.add
                        local.set 22
                        local.get 9
                        local.get 10
                        i32.const 4
                        i32.shl
                        local.tee 11
                        i32.add
                        local.set 21
                        local.get 9
                        local.set 14
                        local.get 16
                        local.set 2
                        local.get 10
                        i32.eqz
                        br_if 7 (;@3;)
                        local.get 9
                        i32.const 16
                        i32.add
                        local.set 14
                        block  ;; label = @11
                          local.get 9
                          i32.load8_u offset=12
                          local.tee 20
                          i32.const 2
                          i32.ne
                          br_if 0 (;@11;)
                          local.get 16
                          local.set 2
                          br 8 (;@3;)
                        end
                        local.get 1
                        i32.load8_u offset=44
                        local.set 19
                        local.get 1
                        i32.load offset=4
                        local.set 12
                        local.get 1
                        i32.load
                        local.set 24
                        local.get 1
                        i32.load offset=24
                        local.set 25
                        local.get 1
                        i32.load offset=20
                        local.set 6
                        local.get 9
                        i64.load align=4
                        local.set 23
                        local.get 4
                        i32.const 128
                        i32.add
                        i32.const 8
                        i32.add
                        local.tee 5
                        local.get 9
                        i32.const 8
                        i32.add
                        i32.load
                        i32.store
                        local.get 9
                        i32.const 15
                        i32.add
                        i32.load8_u
                        local.set 2
                        local.get 4
                        i32.const 141
                        i32.add
                        local.tee 18
                        local.get 9
                        i32.load16_u offset=13 align=1
                        i32.store16 align=1
                        local.get 18
                        i32.const 2
                        i32.add
                        local.get 2
                        i32.store8
                        local.get 4
                        local.get 23
                        i64.store offset=128
                        local.get 4
                        local.get 20
                        i32.store8 offset=140
                        block  ;; label = @11
                          local.get 17
                          br_if 0 (;@11;)
                          local.get 16
                          local.set 2
                          br 7 (;@4;)
                        end
                        local.get 4
                        i32.const 64
                        i32.add
                        i32.const 8
                        i32.add
                        local.get 5
                        i32.load
                        i32.store
                        local.get 4
                        i32.const 60
                        i32.add
                        i32.const 2
                        i32.add
                        local.get 18
                        i32.const 2
                        i32.add
                        i32.load8_u
                        i32.store8
                        local.get 4
                        local.get 4
                        i64.load offset=128
                        i64.store offset=64
                        local.get 4
                        local.get 18
                        i32.load16_u align=1
                        i32.store16 offset=60
                        local.get 16
                        i32.load8_u offset=4
                        local.set 26
                        local.get 16
                        i32.load
                        local.set 27
                        block  ;; label = @11
                          block  ;; label = @12
                            local.get 6
                            br_if 0 (;@12;)
                            i32.const -2147483648
                            local.set 28
                            br 1 (;@11;)
                          end
                          local.get 4
                          i32.const 128
                          i32.add
                          local.get 6
                          i32.const 0
                          local.get 25
                          i32.load offset=16
                          call_indirect (type 3)
                          local.get 4
                          i64.load offset=132 align=4
                          local.set 23
                          local.get 4
                          i32.load offset=128
                          local.set 28
                        end
                        local.get 7
                        local.get 7
                        i32.load
                        local.tee 2
                        i32.const 1
                        i32.add
                        i32.store
                        local.get 2
                        i32.const 0
                        i32.lt_s
                        br_if 4 (;@6;)
                        local.get 16
                        i32.const 8
                        i32.add
                        local.set 2
                        local.get 4
                        i32.const 104
                        i32.add
                        local.tee 5
                        local.get 4
                        i64.load offset=64
                        i64.store align=4
                        local.get 4
                        i32.const 117
                        i32.add
                        local.tee 13
                        local.get 4
                        i32.load16_u offset=60
                        i32.store16 align=1
                        local.get 5
                        i32.const 8
                        i32.add
                        local.get 4
                        i32.const 64
                        i32.add
                        i32.const 8
                        i32.add
                        i32.load
                        i32.store
                        local.get 13
                        i32.const 2
                        i32.add
                        local.get 4
                        i32.const 60
                        i32.add
                        i32.const 2
                        i32.add
                        i32.load8_u
                        i32.store8
                        local.get 4
                        local.get 28
                        i32.store offset=84
                        local.get 4
                        local.get 12
                        i32.store offset=80
                        local.get 4
                        local.get 24
                        i32.store offset=76
                        local.get 4
                        local.get 20
                        i32.store8 offset=116
                        local.get 4
                        i32.const 0
                        i32.store offset=124
                        local.get 4
                        local.get 7
                        i32.store offset=120
                        local.get 4
                        local.get 26
                        i32.store8 offset=100
                        local.get 4
                        local.get 27
                        i32.store offset=96
                        local.get 4
                        local.get 23
                        i64.store offset=88 align=4
                        block  ;; label = @11
                          block  ;; label = @12
                            block  ;; label = @13
                              block  ;; label = @14
                                block  ;; label = @15
                                  block  ;; label = @16
                                    block  ;; label = @17
                                      block  ;; label = @18
                                        block  ;; label = @19
                                          local.get 19
                                          i32.const 1
                                          i32.and
                                          br_if 0 (;@19;)
                                          local.get 4
                                          i32.const 128
                                          i32.add
                                          i32.const 48
                                          i32.add
                                          local.get 4
                                          i32.const 76
                                          i32.add
                                          i32.const 48
                                          i32.add
                                          i32.load
                                          i32.store
                                          local.get 4
                                          i32.const 128
                                          i32.add
                                          i32.const 40
                                          i32.add
                                          local.get 4
                                          i32.const 76
                                          i32.add
                                          i32.const 40
                                          i32.add
                                          i64.load align=4
                                          i64.store
                                          local.get 4
                                          i32.const 128
                                          i32.add
                                          i32.const 32
                                          i32.add
                                          local.get 4
                                          i32.const 76
                                          i32.add
                                          i32.const 32
                                          i32.add
                                          i64.load align=4
                                          i64.store
                                          local.get 4
                                          i32.const 128
                                          i32.add
                                          i32.const 24
                                          i32.add
                                          local.get 4
                                          i32.const 76
                                          i32.add
                                          i32.const 24
                                          i32.add
                                          i64.load align=4
                                          i64.store
                                          local.get 4
                                          i32.const 128
                                          i32.add
                                          i32.const 16
                                          i32.add
                                          local.get 4
                                          i32.const 76
                                          i32.add
                                          i32.const 16
                                          i32.add
                                          i64.load align=4
                                          i64.store
                                          local.get 4
                                          i32.const 128
                                          i32.add
                                          i32.const 8
                                          i32.add
                                          local.get 4
                                          i32.const 76
                                          i32.add
                                          i32.const 8
                                          i32.add
                                          i64.load align=4
                                          i64.store
                                          local.get 4
                                          local.get 4
                                          i64.load offset=76 align=4
                                          i64.store offset=128
                                          local.get 4
                                          i32.const 328
                                          i32.add
                                          local.get 2
                                          local.get 4
                                          i32.const 128
                                          i32.add
                                          call $_ZN88_$LT$rayon_core..registry..DefaultSpawn$u20$as$u20$rayon_core..registry..ThreadSpawn$GT$5spawn17hde45ef0851463ac6E
                                          local.get 4
                                          i32.load8_u offset=328
                                          i32.const 4
                                          i32.eq
                                          br_if 1 (;@18;)
                                          br 8 (;@11;)
                                        end
                                        i32.const 0
                                        i32.load offset=1058996
                                        br_if 1 (;@17;)
                                        local.get 4
                                        i32.const 328
                                        i32.add
                                        i32.const 48
                                        i32.add
                                        local.get 4
                                        i32.const 76
                                        i32.add
                                        i32.const 48
                                        i32.add
                                        i32.load
                                        i32.store
                                        local.get 4
                                        i32.const 328
                                        i32.add
                                        i32.const 40
                                        i32.add
                                        local.get 4
                                        i32.const 76
                                        i32.add
                                        i32.const 40
                                        i32.add
                                        i64.load align=4
                                        i64.store
                                        local.get 4
                                        i32.const 328
                                        i32.add
                                        i32.const 32
                                        i32.add
                                        local.get 4
                                        i32.const 76
                                        i32.add
                                        i32.const 32
                                        i32.add
                                        i64.load align=4
                                        i64.store
                                        local.get 4
                                        i32.const 328
                                        i32.add
                                        i32.const 24
                                        i32.add
                                        local.get 4
                                        i32.const 76
                                        i32.add
                                        i32.const 24
                                        i32.add
                                        i64.load align=4
                                        i64.store
                                        local.get 4
                                        i32.const 328
                                        i32.add
                                        i32.const 16
                                        i32.add
                                        local.get 4
                                        i32.const 76
                                        i32.add
                                        i32.const 16
                                        i32.add
                                        i64.load align=4
                                        i64.store
                                        local.get 4
                                        i32.const 328
                                        i32.add
                                        i32.const 8
                                        i32.add
                                        local.get 4
                                        i32.const 76
                                        i32.add
                                        i32.const 8
                                        i32.add
                                        i64.load align=4
                                        i64.store
                                        local.get 4
                                        local.get 4
                                        i64.load offset=76 align=4
                                        i64.store offset=328
                                        local.get 4
                                        i32.const 128
                                        i32.add
                                        local.get 4
                                        i32.const 328
                                        i32.add
                                        call $_ZN117_$LT$rayon_core..registry..WorkerThread$u20$as$u20$core..convert..From$LT$rayon_core..registry..ThreadBuilder$GT$$GT$4from17h6e861b9dde3d1c34E
                                        i32.const 0
                                        i32.load8_u offset=1058985
                                        drop
                                        i32.const 192
                                        i32.const 64
                                        call $__rust_alloc
                                        local.tee 20
                                        i32.eqz
                                        br_if 3 (;@15;)
                                        local.get 20
                                        local.get 4
                                        i32.const 128
                                        i32.add
                                        i32.const 192
                                        call $memcpy
                                        local.set 20
                                        i32.const 0
                                        i32.load offset=1058996
                                        br_if 2 (;@16;)
                                        i32.const 0
                                        local.get 20
                                        i32.store offset=1058996
                                        local.get 7
                                        i32.load offset=260
                                        i32.eqz
                                        br_if 5 (;@13;)
                                        local.get 7
                                        i32.load offset=256
                                        local.tee 19
                                        i32.load8_u offset=12
                                        local.set 20
                                        local.get 19
                                        i32.const 1
                                        i32.store8 offset=12
                                        local.get 4
                                        local.get 20
                                        i32.store8 offset=328
                                        local.get 20
                                        br_if 4 (;@14;)
                                        local.get 19
                                        i32.const 256
                                        i32.store16 offset=12 align=1
                                      end
                                      local.get 10
                                      i32.const 1
                                      i32.eq
                                      br_if 15 (;@2;)
                                      local.get 11
                                      i32.const -16
                                      i32.add
                                      local.set 28
                                      local.get 17
                                      i32.const 3
                                      i32.shl
                                      i32.const -8
                                      i32.add
                                      local.set 29
                                      i32.const 0
                                      local.set 20
                                      i32.const 1
                                      local.set 2
                                      loop  ;; label = @18
                                        local.get 16
                                        local.get 20
                                        i32.add
                                        local.set 10
                                        block  ;; label = @19
                                          block  ;; label = @20
                                            local.get 14
                                            i32.load8_u offset=12
                                            local.tee 11
                                            i32.const 2
                                            i32.eq
                                            br_if 0 (;@20;)
                                            local.get 14
                                            i64.load align=4
                                            local.set 30
                                            local.get 4
                                            i32.const 128
                                            i32.add
                                            i32.const 8
                                            i32.add
                                            local.tee 19
                                            local.get 14
                                            i32.const 8
                                            i32.add
                                            i32.load
                                            i32.store
                                            local.get 14
                                            i32.const 15
                                            i32.add
                                            i32.load8_u
                                            local.set 17
                                            local.get 18
                                            local.get 14
                                            i32.load16_u offset=13 align=1
                                            i32.store16 align=1
                                            local.get 18
                                            i32.const 2
                                            i32.add
                                            local.tee 26
                                            local.get 17
                                            i32.store8
                                            local.get 4
                                            local.get 30
                                            i64.store offset=128
                                            local.get 4
                                            local.get 11
                                            i32.store8 offset=140
                                            local.get 29
                                            local.get 20
                                            i32.ne
                                            br_if 1 (;@19;)
                                            local.get 14
                                            i32.const 16
                                            i32.add
                                            local.set 14
                                            local.get 22
                                            local.set 2
                                            br 16 (;@4;)
                                          end
                                          local.get 14
                                          i32.const 16
                                          i32.add
                                          local.set 14
                                          local.get 10
                                          i32.const 8
                                          i32.add
                                          local.set 2
                                          br 16 (;@3;)
                                        end
                                        local.get 4
                                        i32.const 64
                                        i32.add
                                        i32.const 8
                                        i32.add
                                        local.tee 27
                                        local.get 19
                                        i32.load
                                        i32.store
                                        local.get 4
                                        i32.const 60
                                        i32.add
                                        i32.const 2
                                        i32.add
                                        local.tee 31
                                        local.get 26
                                        i32.load8_u
                                        i32.store8
                                        local.get 4
                                        local.get 4
                                        i64.load offset=128
                                        i64.store offset=64
                                        local.get 4
                                        local.get 18
                                        i32.load16_u align=1
                                        i32.store16 offset=60
                                        local.get 10
                                        i32.const 8
                                        i32.add
                                        i32.load
                                        local.set 26
                                        local.get 10
                                        i32.const 12
                                        i32.add
                                        i32.load8_u
                                        local.set 32
                                        block  ;; label = @19
                                          block  ;; label = @20
                                            local.get 6
                                            br_if 0 (;@20;)
                                            i32.const -2147483648
                                            local.set 33
                                            br 1 (;@19;)
                                          end
                                          local.get 4
                                          i32.const 128
                                          i32.add
                                          local.get 6
                                          local.get 2
                                          local.get 25
                                          i32.load offset=16
                                          call_indirect (type 3)
                                          local.get 4
                                          i64.load offset=132 align=4
                                          local.set 23
                                          local.get 4
                                          i32.load offset=128
                                          local.set 33
                                        end
                                        local.get 7
                                        local.get 7
                                        i32.load
                                        local.tee 17
                                        i32.const 1
                                        i32.add
                                        i32.store
                                        local.get 17
                                        i32.const -1
                                        i32.le_s
                                        br_if 12 (;@6;)
                                        local.get 4
                                        i32.const 76
                                        i32.add
                                        i32.const 8
                                        i32.add
                                        local.tee 17
                                        local.get 33
                                        i32.store
                                        local.get 5
                                        local.get 4
                                        i64.load offset=64
                                        i64.store align=4
                                        local.get 13
                                        local.get 4
                                        i32.load16_u offset=60
                                        i32.store16 align=1
                                        local.get 4
                                        i32.const 76
                                        i32.add
                                        i32.const 40
                                        i32.add
                                        local.tee 33
                                        local.get 11
                                        i32.store8
                                        local.get 4
                                        i32.const 76
                                        i32.add
                                        i32.const 24
                                        i32.add
                                        local.tee 11
                                        local.get 32
                                        i32.store8
                                        local.get 4
                                        i32.const 76
                                        i32.add
                                        i32.const 48
                                        i32.add
                                        local.get 2
                                        i32.store
                                        local.get 5
                                        i32.const 8
                                        i32.add
                                        local.get 27
                                        i32.load
                                        i32.store
                                        local.get 13
                                        i32.const 2
                                        i32.add
                                        local.get 31
                                        i32.load8_u
                                        i32.store8
                                        local.get 4
                                        local.get 23
                                        i64.store offset=88 align=4
                                        local.get 4
                                        local.get 12
                                        i32.store offset=80
                                        local.get 4
                                        local.get 24
                                        i32.store offset=76
                                        local.get 4
                                        local.get 26
                                        i32.store offset=96
                                        local.get 4
                                        local.get 7
                                        i32.store offset=120
                                        local.get 4
                                        i32.const 128
                                        i32.add
                                        i32.const 48
                                        i32.add
                                        local.get 2
                                        i32.store
                                        local.get 4
                                        i32.const 128
                                        i32.add
                                        i32.const 40
                                        i32.add
                                        local.get 33
                                        i64.load align=4
                                        i64.store
                                        local.get 4
                                        i32.const 128
                                        i32.add
                                        i32.const 24
                                        i32.add
                                        local.get 11
                                        i64.load align=4
                                        i64.store
                                        local.get 4
                                        i32.const 128
                                        i32.add
                                        i32.const 16
                                        i32.add
                                        local.get 4
                                        i32.const 76
                                        i32.add
                                        i32.const 16
                                        i32.add
                                        i64.load align=4
                                        i64.store
                                        local.get 19
                                        local.get 17
                                        i64.load align=4
                                        i64.store
                                        local.get 4
                                        i32.const 128
                                        i32.add
                                        i32.const 32
                                        i32.add
                                        local.get 4
                                        i32.const 76
                                        i32.add
                                        i32.const 32
                                        i32.add
                                        i64.load align=4
                                        i64.store
                                        local.get 4
                                        local.get 4
                                        i64.load offset=76 align=4
                                        i64.store offset=128
                                        local.get 4
                                        i32.const 328
                                        i32.add
                                        local.get 2
                                        local.get 4
                                        i32.const 128
                                        i32.add
                                        call $_ZN88_$LT$rayon_core..registry..DefaultSpawn$u20$as$u20$rayon_core..registry..ThreadSpawn$GT$5spawn17hde45ef0851463ac6E
                                        local.get 4
                                        i32.load8_u offset=328
                                        i32.const 4
                                        i32.ne
                                        br_if 6 (;@12;)
                                        local.get 14
                                        i32.const 16
                                        i32.add
                                        local.set 14
                                        local.get 2
                                        i32.const 1
                                        i32.add
                                        local.set 2
                                        local.get 20
                                        i32.const 8
                                        i32.add
                                        local.set 20
                                        local.get 28
                                        i32.const -16
                                        i32.add
                                        local.tee 28
                                        br_if 0 (;@18;)
                                      end
                                      local.get 16
                                      local.get 20
                                      i32.add
                                      i32.const 8
                                      i32.add
                                      local.set 2
                                      br 15 (;@2;)
                                    end
                                    local.get 0
                                    i64.const 5
                                    i64.store align=4
                                    block  ;; label = @17
                                      local.get 28
                                      i32.const -2147483648
                                      i32.or
                                      i32.const -2147483648
                                      i32.eq
                                      br_if 0 (;@17;)
                                      local.get 23
                                      i32.wrap_i64
                                      local.get 28
                                      i32.const 1
                                      call $__rust_dealloc
                                    end
                                    local.get 4
                                    i32.load offset=104
                                    local.tee 7
                                    local.get 7
                                    i32.load
                                    local.tee 7
                                    i32.const -1
                                    i32.add
                                    i32.store
                                    block  ;; label = @17
                                      local.get 7
                                      i32.const 1
                                      i32.ne
                                      br_if 0 (;@17;)
                                      local.get 5
                                      call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hf1d2b099cab8a99cE
                                    end
                                    local.get 4
                                    i32.load offset=96
                                    local.tee 7
                                    local.get 7
                                    i32.load
                                    local.tee 7
                                    i32.const -1
                                    i32.add
                                    i32.store
                                    block  ;; label = @17
                                      local.get 7
                                      i32.const 1
                                      i32.ne
                                      br_if 0 (;@17;)
                                      local.get 4
                                      i32.const 96
                                      i32.add
                                      call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hf1d2b099cab8a99cE
                                    end
                                    local.get 4
                                    i32.load offset=120
                                    local.tee 7
                                    local.get 7
                                    i32.load
                                    local.tee 7
                                    i32.const -1
                                    i32.add
                                    i32.store
                                    block  ;; label = @17
                                      local.get 7
                                      i32.const 1
                                      i32.ne
                                      br_if 0 (;@17;)
                                      local.get 4
                                      i32.const 120
                                      i32.add
                                      call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h0960bc771bfafaf1E
                                    end
                                    local.get 4
                                    i32.load offset=40
                                    local.set 7
                                    br 11 (;@5;)
                                  end
                                  i32.const 1050288
                                  i32.const 35
                                  i32.const 1050324
                                  call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
                                  unreachable
                                end
                                i32.const 64
                                i32.const 192
                                call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
                                unreachable
                              end
                              local.get 4
                              i64.const 0
                              i64.store offset=140 align=4
                              local.get 4
                              i64.const 17179869185
                              i64.store offset=132 align=4
                              local.get 4
                              i32.const 1050580
                              i32.store offset=128
                              local.get 4
                              i32.const 328
                              i32.add
                              local.get 4
                              i32.const 128
                              i32.add
                              call $_ZN4core9panicking13assert_failed17h9a3e3b4c3cfd5ab0E.llvm.6263538152696972293
                              unreachable
                            end
                            i32.const 0
                            i32.const 0
                            i32.const 1050016
                            call $_ZN4core9panicking18panic_bounds_check17h6c9fc24fb71a0cb6E
                            unreachable
                          end
                          local.get 14
                          i32.const 16
                          i32.add
                          local.set 14
                          local.get 10
                          i32.const 16
                          i32.add
                          local.set 2
                        end
                        local.get 0
                        local.get 4
                        i64.load offset=328
                        i64.store align=4
                        br 5 (;@5;)
                      end
                      local.get 7
                      local.get 14
                      i32.const 1051416
                      call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
                      unreachable
                    end
                    i32.const 64
                    local.get 2
                    i32.const 1051028
                    call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
                    unreachable
                  end
                  i32.const 4
                  i32.const 760
                  call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
                  unreachable
                end
                i32.const 64
                i32.const 320
                call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
              end
              unreachable
            end
            block  ;; label = @5
              local.get 21
              local.get 14
              i32.eq
              br_if 0 (;@5;)
              local.get 21
              local.get 14
              i32.sub
              i32.const 4
              i32.shr_u
              local.set 21
              loop  ;; label = @6
                local.get 14
                i32.load
                local.tee 18
                local.get 18
                i32.load
                local.tee 18
                i32.const -1
                i32.add
                i32.store
                block  ;; label = @7
                  local.get 18
                  i32.const 1
                  i32.ne
                  br_if 0 (;@7;)
                  local.get 14
                  call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hf1d2b099cab8a99cE
                end
                local.get 14
                i32.const 16
                i32.add
                local.set 14
                local.get 21
                i32.const -1
                i32.add
                local.tee 21
                br_if 0 (;@6;)
              end
            end
            block  ;; label = @5
              local.get 8
              i32.eqz
              br_if 0 (;@5;)
              local.get 9
              local.get 8
              i32.const 4
              i32.shl
              i32.const 4
              call $__rust_dealloc
            end
            block  ;; label = @5
              local.get 22
              local.get 2
              i32.eq
              br_if 0 (;@5;)
              local.get 22
              local.get 2
              i32.sub
              i32.const 3
              i32.shr_u
              local.set 14
              loop  ;; label = @6
                local.get 2
                i32.load
                local.tee 21
                local.get 21
                i32.load
                local.tee 21
                i32.const -1
                i32.add
                i32.store
                block  ;; label = @7
                  local.get 21
                  i32.const 1
                  i32.ne
                  br_if 0 (;@7;)
                  local.get 2
                  call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hf1d2b099cab8a99cE
                end
                local.get 2
                i32.const 8
                i32.add
                local.set 2
                local.get 14
                i32.const -1
                i32.add
                local.tee 14
                br_if 0 (;@6;)
              end
            end
            block  ;; label = @5
              local.get 15
              i32.eqz
              br_if 0 (;@5;)
              local.get 16
              local.get 15
              i32.const 3
              i32.shl
              i32.const 4
              call $__rust_dealloc
            end
            local.get 7
            local.get 7
            i32.load offset=232
            local.tee 2
            i32.const -1
            i32.add
            i32.store offset=232
            block  ;; label = @5
              local.get 2
              i32.const 1
              i32.ne
              br_if 0 (;@5;)
              local.get 7
              i32.load offset=260
              local.tee 2
              i32.eqz
              br_if 0 (;@5;)
              local.get 2
              i32.const 4
              i32.shl
              local.set 18
              local.get 7
              i32.load offset=256
              i32.const 8
              i32.add
              local.set 2
              i32.const 0
              local.set 21
              i32.const 0
              local.set 14
              loop  ;; label = @6
                local.get 2
                i32.load
                local.set 22
                local.get 2
                i32.const 3
                i32.store
                block  ;; label = @7
                  local.get 22
                  i32.const 2
                  i32.ne
                  br_if 0 (;@7;)
                  block  ;; label = @8
                    local.get 7
                    i32.load offset=244
                    local.tee 22
                    local.get 14
                    i32.le_u
                    br_if 0 (;@8;)
                    local.get 4
                    local.get 7
                    i32.load offset=240
                    local.get 21
                    i32.add
                    local.tee 22
                    i32.load8_u
                    local.tee 20
                    i32.store8 offset=328
                    local.get 22
                    i32.const 1
                    i32.store8
                    block  ;; label = @9
                      local.get 20
                      i32.const 1
                      i32.eq
                      br_if 0 (;@9;)
                      block  ;; label = @10
                        local.get 22
                        i32.const 1
                        i32.add
                        local.tee 20
                        i32.load8_u
                        i32.eqz
                        br_if 0 (;@10;)
                        local.get 20
                        i32.const 0
                        i32.store8
                        local.get 7
                        local.get 7
                        i32.load offset=248
                        i32.const -1
                        i32.add
                        i32.store offset=248
                      end
                      local.get 22
                      i32.const 0
                      i32.store8
                      br 2 (;@7;)
                    end
                    local.get 4
                    i64.const 0
                    i64.store offset=140 align=4
                    local.get 4
                    i64.const 17179869185
                    i64.store offset=132 align=4
                    local.get 4
                    i32.const 1050580
                    i32.store offset=128
                    local.get 4
                    i32.const 328
                    i32.add
                    local.get 4
                    i32.const 128
                    i32.add
                    call $_ZN4core9panicking13assert_failed17h9a3e3b4c3cfd5ab0E.llvm.6263538152696972293
                    unreachable
                  end
                  local.get 14
                  local.get 22
                  i32.const 1051060
                  call $_ZN4core9panicking18panic_bounds_check17h6c9fc24fb71a0cb6E
                  unreachable
                end
                local.get 14
                i32.const 1
                i32.add
                local.set 14
                local.get 21
                i32.const 64
                i32.add
                local.set 21
                local.get 2
                i32.const 16
                i32.add
                local.set 2
                local.get 18
                i32.const -16
                i32.add
                local.tee 18
                br_if 0 (;@6;)
              end
            end
            local.get 4
            i32.load offset=40
            local.tee 2
            local.get 2
            i32.load
            local.tee 2
            i32.const -1
            i32.add
            i32.store
            local.get 2
            i32.const 1
            i32.ne
            br_if 3 (;@1;)
            local.get 4
            i32.const 40
            i32.add
            call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h0960bc771bfafaf1E
            br 3 (;@1;)
          end
          local.get 4
          i32.load offset=128
          local.tee 7
          local.get 7
          i32.load
          local.tee 7
          i32.const -1
          i32.add
          i32.store
          local.get 7
          i32.const 1
          i32.ne
          br_if 0 (;@3;)
          local.get 4
          i32.const 128
          i32.add
          call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hf1d2b099cab8a99cE
        end
        local.get 21
        local.get 14
        i32.eq
        br_if 0 (;@2;)
        local.get 21
        local.get 14
        i32.sub
        i32.const 4
        i32.shr_u
        local.set 7
        loop  ;; label = @3
          local.get 14
          i32.load
          local.tee 21
          local.get 21
          i32.load
          local.tee 21
          i32.const -1
          i32.add
          i32.store
          block  ;; label = @4
            local.get 21
            i32.const 1
            i32.ne
            br_if 0 (;@4;)
            local.get 14
            call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hf1d2b099cab8a99cE
          end
          local.get 14
          i32.const 16
          i32.add
          local.set 14
          local.get 7
          i32.const -1
          i32.add
          local.tee 7
          br_if 0 (;@3;)
        end
      end
      block  ;; label = @2
        local.get 8
        i32.eqz
        br_if 0 (;@2;)
        local.get 9
        local.get 8
        i32.const 4
        i32.shl
        i32.const 4
        call $__rust_dealloc
      end
      block  ;; label = @2
        local.get 22
        local.get 2
        i32.eq
        br_if 0 (;@2;)
        local.get 22
        local.get 2
        i32.sub
        i32.const 3
        i32.shr_u
        local.set 14
        loop  ;; label = @3
          local.get 2
          i32.load
          local.tee 7
          local.get 7
          i32.load
          local.tee 7
          i32.const -1
          i32.add
          i32.store
          block  ;; label = @4
            local.get 7
            i32.const 1
            i32.ne
            br_if 0 (;@4;)
            local.get 2
            call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hf1d2b099cab8a99cE
          end
          local.get 2
          i32.const 8
          i32.add
          local.set 2
          local.get 14
          i32.const -1
          i32.add
          local.tee 14
          br_if 0 (;@3;)
        end
      end
      block  ;; label = @2
        local.get 15
        i32.eqz
        br_if 0 (;@2;)
        local.get 16
        local.get 15
        i32.const 3
        i32.shl
        i32.const 4
        call $__rust_dealloc
      end
      local.get 0
      local.get 4
      i32.load offset=40
      i32.store offset=4
      local.get 0
      i32.const 6
      i32.store8
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=12
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 1
        i32.load offset=16
        local.tee 14
        i32.load
        local.tee 4
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 4
        call_indirect (type 0)
      end
      local.get 14
      i32.load offset=4
      local.tee 4
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 4
      local.get 14
      i32.load offset=8
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=20
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 1
        i32.load offset=24
        local.tee 14
        i32.load
        local.tee 4
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 4
        call_indirect (type 0)
      end
      local.get 14
      i32.load offset=4
      local.tee 4
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 4
      local.get 14
      i32.load offset=8
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=28
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 1
        i32.load offset=32
        local.tee 14
        i32.load
        local.tee 4
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 4
        call_indirect (type 0)
      end
      local.get 14
      i32.load offset=4
      local.tee 4
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 4
      local.get 14
      i32.load offset=8
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=36
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 1
        i32.load offset=40
        local.tee 14
        i32.load
        local.tee 4
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 4
        call_indirect (type 0)
      end
      local.get 14
      i32.load offset=4
      local.tee 4
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 4
      local.get 14
      i32.load offset=8
      call $__rust_dealloc
    end
    local.get 3
    global.set $__stack_pointer)
  (func $_ZN10rayon_core8registry8Registry6inject17h51e937ac645609bdE (type 3) (param i32 i32 i32)
    (local i32 i32)
    local.get 0
    i32.load offset=64
    local.set 3
    local.get 0
    i32.load
    local.set 4
    local.get 0
    local.get 1
    local.get 2
    call $_ZN15crossbeam_deque5deque17Injector$LT$T$GT$4push17h9e2e7a20ad51dc77E
    local.get 3
    local.get 4
    i32.xor
    local.set 4
    block  ;; label = @1
      loop  ;; label = @2
        block  ;; label = @3
          local.get 0
          i32.load offset=184
          local.tee 2
          i32.const 65536
          i32.and
          i32.eqz
          br_if 0 (;@3;)
          local.get 2
          local.set 3
          br 2 (;@1;)
        end
        local.get 0
        local.get 2
        i32.const 65536
        i32.or
        local.tee 3
        local.get 0
        i32.load offset=184
        local.tee 1
        local.get 1
        local.get 2
        i32.eq
        select
        i32.store offset=184
        local.get 1
        local.get 2
        i32.ne
        br_if 0 (;@2;)
      end
    end
    block  ;; label = @1
      local.get 3
      i32.const 255
      i32.and
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 4
        i32.const 1
        i32.gt_u
        br_if 0 (;@2;)
        local.get 3
        i32.const 8
        i32.shr_u
        i32.const 255
        i32.and
        local.get 2
        i32.ne
        br_if 1 (;@1;)
      end
      local.get 0
      i32.const 172
      i32.add
      i32.const 1
      call $_ZN10rayon_core5sleep5Sleep16wake_any_threads17h17244bbca7ed393dE
    end)
  (func $_ZN10rayon_core8registry8Registry26notify_worker_latch_is_set17h0c173997f1598668E (type 2) (param i32 i32)
    (local i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      local.get 0
      i32.load offset=180
      local.tee 3
      local.get 1
      i32.le_u
      br_if 0 (;@1;)
      local.get 2
      local.get 0
      i32.load offset=176
      local.get 1
      i32.const 6
      i32.shl
      i32.add
      local.tee 1
      i32.load8_u
      local.tee 3
      i32.store8 offset=7
      local.get 1
      i32.const 1
      i32.store8
      block  ;; label = @2
        local.get 3
        i32.const 1
        i32.eq
        br_if 0 (;@2;)
        block  ;; label = @3
          local.get 1
          i32.load8_u offset=1
          i32.eqz
          br_if 0 (;@3;)
          local.get 1
          i32.const 0
          i32.store8 offset=1
          local.get 0
          local.get 0
          i32.load offset=184
          i32.const -1
          i32.add
          i32.store offset=184
        end
        local.get 1
        i32.const 0
        i32.store8
        local.get 2
        i32.const 32
        i32.add
        global.set $__stack_pointer
        return
      end
      local.get 2
      i64.const 0
      i64.store offset=20 align=4
      local.get 2
      i64.const 17179869185
      i64.store offset=12 align=4
      local.get 2
      i32.const 1050580
      i32.store offset=8
      local.get 2
      i32.const 7
      i32.add
      local.get 2
      i32.const 8
      i32.add
      call $_ZN4core9panicking13assert_failed17h9a3e3b4c3cfd5ab0E.llvm.6263538152696972293
      unreachable
    end
    local.get 1
    local.get 3
    i32.const 1051060
    call $_ZN4core9panicking18panic_bounds_check17h6c9fc24fb71a0cb6E
    unreachable)
  (func $_ZN69_$LT$rayon_core..ThreadPoolBuildError$u20$as$u20$core..fmt..Debug$GT$3fmt17h3c8116c51c896825E.llvm.16897747005057573272 (type 4) (param i32 i32) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    local.get 0
    i32.store offset=12
    local.get 1
    i32.const 1050096
    i32.const 20
    i32.const 1050116
    i32.const 4
    local.get 2
    i32.const 12
    i32.add
    i32.const 1050080
    call $_ZN4core3fmt9Formatter26debug_struct_field1_finish17ha7586c40638b50f9E
    local.set 0
    local.get 2
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN4core4iter6traits8iterator8Iterator5unzip17h1f17632128fc2d60E (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i64 i64)
    global.get $__stack_pointer
    i32.const 64
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i64.const 4
    i64.store offset=24 align=4
    local.get 2
    i64.const 0
    i64.store offset=16 align=4
    local.get 2
    i64.const 17179869184
    i64.store offset=8 align=4
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.load offset=8
        local.tee 3
        local.get 1
        i32.load offset=4
        local.tee 4
        i32.le_u
        br_if 0 (;@2;)
        local.get 2
        i32.const 8
        i32.add
        i32.const 0
        i32.const 0
        local.get 3
        local.get 4
        i32.sub
        local.tee 5
        local.get 5
        local.get 3
        i32.gt_u
        select
        local.tee 3
        i32.const 4
        i32.const 16
        call $_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17h7844c62910e0bc64E
        block  ;; label = @3
          block  ;; label = @4
            local.get 2
            i32.load offset=20
            local.get 2
            i32.load offset=28
            local.tee 6
            i32.sub
            local.get 3
            i32.lt_u
            br_if 0 (;@4;)
            i32.const 4
            local.set 4
            br 1 (;@3;)
          end
          local.get 2
          i32.const 20
          i32.add
          local.get 6
          local.get 3
          i32.const 4
          i32.const 8
          call $_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17h7844c62910e0bc64E
          local.get 2
          i32.load offset=24
          local.set 4
          local.get 2
          i32.load offset=28
          local.set 6
        end
        local.get 2
        i32.load offset=12
        local.get 2
        i32.load offset=16
        local.tee 7
        i32.const 4
        i32.shl
        i32.add
        local.set 3
        local.get 4
        local.get 6
        i32.const 3
        i32.shl
        i32.add
        local.set 4
        local.get 1
        i32.load
        local.set 8
        loop  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 8
              i32.load8_u
              br_if 0 (;@5;)
              local.get 2
              i32.const 48
              i32.add
              call $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$8new_lifo17hae9d1ea4214d0b5bE
              br 1 (;@4;)
            end
            local.get 2
            i32.const 48
            i32.add
            call $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$8new_fifo17had5b41e2b64bb3a1E
          end
          local.get 2
          i32.load offset=48
          local.tee 1
          local.get 1
          i32.load
          local.tee 9
          i32.const 1
          i32.add
          i32.store
          local.get 9
          i32.const -1
          i32.le_s
          br_if 2 (;@1;)
          local.get 2
          i32.const 32
          i32.add
          i32.const 8
          i32.add
          local.get 2
          i32.const 48
          i32.add
          i32.const 8
          i32.add
          i64.load align=4
          local.tee 10
          i64.store
          local.get 2
          local.get 2
          i64.load offset=48 align=4
          local.tee 11
          i64.store offset=32
          local.get 2
          i32.load8_u offset=60
          local.set 9
          local.get 3
          i32.const 8
          i32.add
          local.get 10
          i64.store align=4
          local.get 3
          local.get 11
          i64.store align=4
          local.get 4
          i32.const 4
          i32.add
          local.get 9
          i32.store8
          local.get 4
          local.get 1
          i32.store
          local.get 3
          i32.const 16
          i32.add
          local.set 3
          local.get 4
          i32.const 8
          i32.add
          local.set 4
          local.get 6
          i32.const 1
          i32.add
          local.set 6
          local.get 7
          i32.const 1
          i32.add
          local.set 7
          local.get 5
          i32.const -1
          i32.add
          local.tee 5
          br_if 0 (;@3;)
        end
        local.get 2
        local.get 6
        i32.store offset=28
        local.get 2
        local.get 7
        i32.store offset=16
      end
      local.get 0
      local.get 2
      i64.load offset=8 align=4
      i64.store align=4
      local.get 0
      i32.const 16
      i32.add
      local.get 2
      i32.const 8
      i32.add
      i32.const 16
      i32.add
      i64.load align=4
      i64.store align=4
      local.get 0
      i32.const 8
      i32.add
      local.get 2
      i32.const 8
      i32.add
      i32.const 8
      i32.add
      i64.load align=4
      i64.store align=4
      local.get 2
      i32.const 64
      i32.add
      global.set $__stack_pointer
      return
    end
    unreachable)
  (func $_ZN4core4iter6traits8iterator8Iterator5unzip17he9948bad0b92d0b4E (type 3) (param i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32 i64 i64)
    global.get $__stack_pointer
    i32.const 64
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 3
    i64.const 4
    i64.store offset=24 align=4
    local.get 3
    i64.const 0
    i64.store offset=16 align=4
    local.get 3
    i64.const 17179869184
    i64.store offset=8 align=4
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        local.get 1
        i32.le_u
        br_if 0 (;@2;)
        local.get 3
        i32.const 8
        i32.add
        i32.const 0
        i32.const 0
        local.get 2
        local.get 1
        i32.sub
        local.tee 4
        local.get 4
        local.get 2
        i32.gt_u
        select
        local.tee 2
        i32.const 4
        i32.const 16
        call $_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17h7844c62910e0bc64E
        block  ;; label = @3
          block  ;; label = @4
            local.get 3
            i32.load offset=20
            local.get 3
            i32.load offset=28
            local.tee 5
            i32.sub
            local.get 2
            i32.lt_u
            br_if 0 (;@4;)
            i32.const 4
            local.set 1
            br 1 (;@3;)
          end
          local.get 3
          i32.const 20
          i32.add
          local.get 5
          local.get 2
          i32.const 4
          i32.const 8
          call $_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17h7844c62910e0bc64E
          local.get 3
          i32.load offset=28
          local.set 5
          local.get 3
          i32.load offset=24
          local.set 1
        end
        local.get 3
        i32.load offset=12
        local.get 3
        i32.load offset=16
        local.tee 6
        i32.const 4
        i32.shl
        i32.add
        local.set 2
        local.get 1
        local.get 5
        i32.const 3
        i32.shl
        i32.add
        local.set 1
        loop  ;; label = @3
          local.get 3
          i32.const 48
          i32.add
          call $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$8new_fifo17had5b41e2b64bb3a1E
          local.get 3
          i32.load offset=48
          local.tee 7
          local.get 7
          i32.load
          local.tee 8
          i32.const 1
          i32.add
          i32.store
          local.get 8
          i32.const -1
          i32.le_s
          br_if 2 (;@1;)
          local.get 3
          i32.const 32
          i32.add
          i32.const 8
          i32.add
          local.get 3
          i32.const 48
          i32.add
          i32.const 8
          i32.add
          i64.load align=4
          local.tee 9
          i64.store
          local.get 3
          local.get 3
          i64.load offset=48 align=4
          local.tee 10
          i64.store offset=32
          local.get 3
          i32.load8_u offset=60
          local.set 8
          local.get 2
          i32.const 8
          i32.add
          local.get 9
          i64.store align=4
          local.get 2
          local.get 10
          i64.store align=4
          local.get 1
          i32.const 4
          i32.add
          local.get 8
          i32.store8
          local.get 1
          local.get 7
          i32.store
          local.get 2
          i32.const 16
          i32.add
          local.set 2
          local.get 1
          i32.const 8
          i32.add
          local.set 1
          local.get 5
          i32.const 1
          i32.add
          local.set 5
          local.get 6
          i32.const 1
          i32.add
          local.set 6
          local.get 4
          i32.const -1
          i32.add
          local.tee 4
          br_if 0 (;@3;)
        end
        local.get 3
        local.get 5
        i32.store offset=28
        local.get 3
        local.get 6
        i32.store offset=16
      end
      local.get 0
      local.get 3
      i64.load offset=8 align=4
      i64.store align=4
      local.get 0
      i32.const 16
      i32.add
      local.get 3
      i32.const 8
      i32.add
      i32.const 16
      i32.add
      i64.load align=4
      i64.store align=4
      local.get 0
      i32.const 8
      i32.add
      local.get 3
      i32.const 8
      i32.add
      i32.const 8
      i32.add
      i64.load align=4
      i64.store align=4
      local.get 3
      i32.const 64
      i32.add
      global.set $__stack_pointer
      return
    end
    unreachable)
  (func $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h00ddb978cc2c31d9E (type 0) (param i32)
    (local i32 i32)
    block  ;; label = @1
      local.get 0
      i32.load
      local.tee 0
      i32.const 16
      i32.add
      i32.load
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.const 20
      i32.add
      i32.load
      local.set 2
      local.get 1
      i32.const 0
      i32.store8
      local.get 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.get 2
      i32.const 1
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 0
      i32.const -1
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      local.get 0
      i32.load offset=4
      local.tee 1
      i32.const -1
      i32.add
      i32.store offset=4
      local.get 1
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 24
      i32.const 8
      call $__rust_dealloc
    end)
  (func $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h0960bc771bfafaf1E (type 0) (param i32)
    (local i32 i32 i32 i32)
    block  ;; label = @1
      local.get 0
      i32.load
      local.tee 1
      i32.load offset=260
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.load offset=256
      local.set 0
      loop  ;; label = @2
        local.get 0
        i32.load
        local.tee 3
        local.get 3
        i32.load
        local.tee 3
        i32.const -1
        i32.add
        i32.store
        block  ;; label = @3
          local.get 3
          i32.const 1
          i32.ne
          br_if 0 (;@3;)
          local.get 0
          call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hf1d2b099cab8a99cE
        end
        local.get 0
        i32.const 16
        i32.add
        local.set 0
        local.get 2
        i32.const -1
        i32.add
        local.tee 2
        br_if 0 (;@2;)
      end
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=252
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.load offset=256
      local.get 0
      i32.const 4
      i32.shl
      i32.const 4
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=236
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.load offset=240
      local.get 0
      i32.const 6
      i32.shl
      i32.const 64
      call $__rust_dealloc
    end
    local.get 1
    i32.load offset=68
    local.set 2
    block  ;; label = @1
      local.get 1
      i32.load offset=64
      i32.const -2
      i32.and
      local.tee 0
      local.get 1
      i32.load offset=128
      i32.const -2
      i32.and
      local.tee 3
      i32.eq
      br_if 0 (;@1;)
      loop  ;; label = @2
        block  ;; label = @3
          local.get 0
          i32.const 126
          i32.and
          i32.const 126
          i32.ne
          br_if 0 (;@3;)
          local.get 2
          i32.load
          local.set 4
          local.get 2
          i32.const 760
          i32.const 4
          call $__rust_dealloc
          local.get 4
          local.set 2
        end
        local.get 3
        local.get 0
        i32.const 2
        i32.add
        local.tee 0
        i32.ne
        br_if 0 (;@2;)
      end
    end
    local.get 2
    i32.const 760
    i32.const 4
    call $__rust_dealloc
    block  ;; label = @1
      local.get 1
      i32.load offset=204
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.load offset=200
      local.set 0
      loop  ;; label = @2
        local.get 0
        i32.load
        local.tee 3
        local.get 3
        i32.load
        local.tee 3
        i32.const -1
        i32.add
        i32.store
        block  ;; label = @3
          local.get 3
          i32.const 1
          i32.ne
          br_if 0 (;@3;)
          local.get 0
          call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hf1d2b099cab8a99cE
        end
        local.get 0
        i32.const 16
        i32.add
        local.set 0
        local.get 2
        i32.const -1
        i32.add
        local.tee 2
        br_if 0 (;@2;)
      end
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=196
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.load offset=200
      local.get 0
      i32.const 4
      i32.shl
      i32.const 4
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=208
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 1
        i32.load offset=212
        local.tee 2
        i32.load
        local.tee 3
        i32.eqz
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        call_indirect (type 0)
      end
      local.get 2
      i32.load offset=4
      local.tee 3
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 3
      local.get 2
      i32.load offset=8
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=216
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 1
        i32.load offset=220
        local.tee 2
        i32.load
        local.tee 3
        i32.eqz
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        call_indirect (type 0)
      end
      local.get 2
      i32.load offset=4
      local.tee 3
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 3
      local.get 2
      i32.load offset=8
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 1
      i32.load offset=224
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 1
        i32.load offset=228
        local.tee 2
        i32.load
        local.tee 3
        i32.eqz
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        call_indirect (type 0)
      end
      local.get 2
      i32.load offset=4
      local.tee 3
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 3
      local.get 2
      i32.load offset=8
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 1
      i32.const -1
      i32.eq
      br_if 0 (;@1;)
      local.get 1
      local.get 1
      i32.load offset=4
      local.tee 0
      i32.const -1
      i32.add
      i32.store offset=4
      local.get 0
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 1
      i32.const 320
      i32.const 64
      call $__rust_dealloc
    end)
  (func $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hf1d2b099cab8a99cE (type 0) (param i32)
    (local i32 i32)
    block  ;; label = @1
      local.get 0
      i32.load
      local.tee 0
      i32.load offset=64
      i32.const -4
      i32.and
      local.tee 1
      i32.load offset=4
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.load
      local.get 2
      i32.const 3
      i32.shl
      i32.const 4
      call $__rust_dealloc
    end
    local.get 1
    i32.const 8
    i32.const 4
    call $__rust_dealloc
    block  ;; label = @1
      local.get 0
      i32.const -1
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      local.get 0
      i32.load offset=4
      local.tee 1
      i32.const -1
      i32.add
      i32.store offset=4
      local.get 1
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 192
      i32.const 64
      call $__rust_dealloc
    end)
  (func $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h39754dcdb290b379E (type 0) (param i32)
    (local i32)
    block  ;; label = @1
      local.get 0
      i32.load offset=8
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.load offset=12
      local.tee 1
      local.get 1
      i32.load
      local.tee 1
      i32.const -1
      i32.add
      i32.store
      local.get 1
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 12
      i32.add
      call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h00ddb978cc2c31d9E
    end
    block  ;; label = @1
      local.get 0
      i32.const -1
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      local.get 0
      i32.load offset=4
      local.tee 1
      i32.const -1
      i32.add
      i32.store offset=4
      local.get 1
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 24
      i32.const 4
      call $__rust_dealloc
    end)
  (func $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h4e00d7284bb1d392E (type 0) (param i32)
    (local i32 i32 i32)
    local.get 0
    i32.load
    local.tee 0
    i32.load offset=8
    local.set 1
    block  ;; label = @1
      local.get 0
      i32.const 12
      i32.add
      i32.load
      local.tee 2
      i32.load
      local.tee 3
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.get 3
      call_indirect (type 0)
    end
    block  ;; label = @1
      local.get 2
      i32.load offset=4
      local.tee 3
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.get 3
      local.get 2
      i32.load offset=8
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 0
      i32.load offset=16
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 2
      i32.load
      local.tee 1
      i32.const -1
      i32.add
      i32.store
      local.get 1
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 16
      i32.add
      call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h4e00d7284bb1d392E
    end
    block  ;; label = @1
      local.get 0
      i32.const -1
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      local.get 0
      i32.load offset=4
      local.tee 2
      i32.const -1
      i32.add
      i32.store offset=4
      local.get 2
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 20
      i32.const 4
      call $__rust_dealloc
    end)
  (func $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h5a9d89913051a5edE (type 0) (param i32)
    (local i32 i32 i32 i32)
    local.get 0
    i32.load
    local.tee 0
    i32.load offset=16
    local.set 1
    block  ;; label = @1
      local.get 0
      i32.load offset=12
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 0
        i32.load offset=20
        local.tee 3
        i32.load
        local.tee 4
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        local.get 4
        call_indirect (type 0)
      end
      local.get 3
      i32.load offset=4
      local.tee 4
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.get 4
      local.get 3
      i32.load offset=8
      call $__rust_dealloc
    end
    local.get 0
    i32.const 0
    i32.store offset=12
    block  ;; label = @1
      local.get 0
      i32.load offset=8
      local.tee 3
      i32.eqz
      br_if 0 (;@1;)
      local.get 3
      i32.const 8
      i32.add
      local.get 2
      local.get 1
      i32.const 0
      i32.ne
      i32.and
      call $_ZN3std6thread6scoped9ScopeData29decrement_num_running_threads17hde25e83d0184f65dE
      block  ;; label = @2
        local.get 0
        i32.load offset=8
        local.tee 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        local.get 1
        i32.load
        local.tee 2
        i32.const -1
        i32.add
        i32.store
        local.get 2
        i32.const 1
        i32.ne
        br_if 0 (;@2;)
        local.get 0
        i32.load offset=8
        call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h39754dcdb290b379E
      end
      local.get 0
      i32.load offset=12
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.load offset=16
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 0
        i32.load offset=20
        local.tee 2
        i32.load
        local.tee 3
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        local.get 3
        call_indirect (type 0)
      end
      local.get 2
      i32.load offset=4
      local.tee 3
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.get 3
      local.get 2
      i32.load offset=8
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 0
      i32.const -1
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      local.get 0
      i32.load offset=4
      local.tee 1
      i32.const -1
      i32.add
      i32.store offset=4
      local.get 1
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 24
      i32.const 4
      call $__rust_dealloc
    end)
  (func $_ZN106_$LT$core..iter..adapters..chain..Chain$LT$A$C$B$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$8try_fold17h8f59e602544a6002E (type 3) (param i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.load
        i32.const 1
        i32.ne
        br_if 0 (;@2;)
        local.get 1
        i32.load offset=8
        local.tee 4
        local.get 1
        i32.load offset=4
        local.tee 5
        local.get 4
        local.get 5
        i32.gt_u
        select
        local.set 6
        local.get 2
        i32.load offset=4
        local.get 5
        i32.const 4
        i32.shl
        i32.add
        local.set 7
        local.get 2
        i32.load offset=12
        local.set 8
        local.get 2
        i32.load offset=8
        local.set 9
        local.get 2
        i32.load
        local.set 10
        block  ;; label = @3
          loop  ;; label = @4
            local.get 6
            local.get 5
            i32.eq
            br_if 1 (;@3;)
            local.get 1
            local.get 5
            i32.const 1
            i32.add
            local.tee 11
            i32.store offset=4
            i32.const 0
            local.set 4
            block  ;; label = @5
              local.get 5
              local.get 10
              i32.load
              i32.load offset=136
              i32.eq
              br_if 0 (;@5;)
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 9
                    local.get 5
                    i32.le_u
                    br_if 0 (;@8;)
                    local.get 3
                    i32.const 4
                    i32.add
                    local.get 7
                    call $_ZN15crossbeam_deque5deque16Stealer$LT$T$GT$5steal17hb6210202028f7ff8E
                    i32.const 0
                    local.set 4
                    local.get 3
                    i32.load offset=4
                    br_table 3 (;@5;) 1 (;@7;) 2 (;@6;) 3 (;@5;)
                  end
                  local.get 5
                  local.get 9
                  i32.const 1050820
                  call $_ZN4core9panicking18panic_bounds_check17h6c9fc24fb71a0cb6E
                  unreachable
                end
                local.get 3
                i32.load offset=12
                local.set 12
                local.get 3
                i32.load offset=8
                local.set 4
                br 1 (;@5;)
              end
              local.get 8
              i32.const 1
              i32.store8
            end
            local.get 7
            i32.const 16
            i32.add
            local.set 7
            local.get 11
            local.set 5
            local.get 4
            i32.eqz
            br_if 0 (;@4;)
            br 3 (;@1;)
          end
        end
        local.get 1
        i32.const 0
        i32.store
      end
      block  ;; label = @2
        local.get 1
        i32.load offset=12
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.load offset=20
        local.tee 4
        local.get 1
        i32.load offset=16
        local.tee 5
        local.get 4
        local.get 5
        i32.gt_u
        select
        local.set 6
        local.get 2
        i32.load offset=4
        local.get 5
        i32.const 4
        i32.shl
        i32.add
        local.set 7
        local.get 2
        i32.load offset=12
        local.set 8
        local.get 2
        i32.load offset=8
        local.set 9
        local.get 2
        i32.load
        local.set 10
        loop  ;; label = @3
          local.get 6
          local.get 5
          i32.eq
          br_if 1 (;@2;)
          local.get 1
          local.get 5
          i32.const 1
          i32.add
          local.tee 11
          i32.store offset=16
          i32.const 0
          local.set 4
          block  ;; label = @4
            local.get 5
            local.get 10
            i32.load
            i32.load offset=136
            i32.eq
            br_if 0 (;@4;)
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  local.get 9
                  local.get 5
                  i32.le_u
                  br_if 0 (;@7;)
                  local.get 3
                  i32.const 4
                  i32.add
                  local.get 7
                  call $_ZN15crossbeam_deque5deque16Stealer$LT$T$GT$5steal17hb6210202028f7ff8E
                  i32.const 0
                  local.set 4
                  local.get 3
                  i32.load offset=4
                  br_table 3 (;@4;) 1 (;@6;) 2 (;@5;) 3 (;@4;)
                end
                local.get 5
                local.get 9
                i32.const 1050820
                call $_ZN4core9panicking18panic_bounds_check17h6c9fc24fb71a0cb6E
                unreachable
              end
              local.get 3
              i32.load offset=12
              local.set 12
              local.get 3
              i32.load offset=8
              local.set 4
              br 1 (;@4;)
            end
            local.get 8
            i32.const 1
            i32.store8
          end
          local.get 7
          i32.const 16
          i32.add
          local.set 7
          local.get 11
          local.set 5
          local.get 4
          br_if 2 (;@1;)
          br 0 (;@3;)
        end
      end
      i32.const 0
      local.set 4
    end
    local.get 0
    local.get 12
    i32.store offset=4
    local.get 0
    local.get 4
    i32.store
    local.get 3
    i32.const 16
    i32.add
    global.set $__stack_pointer)
  (func $_ZN15crossbeam_epoch8deferred8Deferred3new4call17h2992095b69a7b6f4E.llvm.6263538152696972293 (type 0) (param i32)
    (local i32)
    block  ;; label = @1
      local.get 0
      i32.load
      i32.const -4
      i32.and
      local.tee 0
      i32.load offset=4
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.load
      local.get 1
      i32.const 3
      i32.shl
      i32.const 4
      call $__rust_dealloc
    end
    local.get 0
    i32.const 8
    i32.const 4
    call $__rust_dealloc)
  (func $_ZN3std3sys12thread_local6statik20LazyStorage$LT$T$GT$10initialize17heb39f68f3dc1c612E (type 4) (param i32 i32) (result i32)
    (local i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.load
        local.set 2
        local.get 1
        i32.const 0
        i32.store
        local.get 2
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.load offset=4
        local.set 2
        br 1 (;@1;)
      end
      call $_ZN15crossbeam_epoch7default17default_collector17h7fa099018adad5efE
      call $_ZN15crossbeam_epoch9collector9Collector8register17h6bed026672a5d855E
      local.set 2
    end
    local.get 0
    i32.load offset=4
    local.set 1
    local.get 0
    local.get 2
    i32.store offset=4
    local.get 0
    i32.load
    local.set 2
    local.get 0
    i32.const 1
    i32.store
    local.get 0
    i32.const 4
    i32.add
    local.set 0
    block  ;; label = @1
      local.get 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.get 1
      i32.load offset=1040
      local.tee 2
      i32.const -1
      i32.add
      i32.store offset=1040
      local.get 1
      i32.load offset=1036
      br_if 0 (;@1;)
      local.get 2
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 1
      call $_ZN15crossbeam_epoch8internal5Local8finalize17ha822a731c89268d9E
    end
    local.get 0)
  (func $_ZN3std3sys4sync4once10no_threads4Once4call17h8c7d0a8c72d63a8fE (type 12) (param i32 i32 i32 i32)
    (local i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 4
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                local.get 0
                i32.load8_u
                br_table 1 (;@5;) 0 (;@6;) 5 (;@1;) 2 (;@4;) 1 (;@5;)
              end
              local.get 1
              i32.eqz
              br_if 2 (;@3;)
            end
            local.get 0
            i32.const 2
            i32.store8
            local.get 2
            i32.load
            local.tee 1
            i32.load
            local.set 2
            local.get 1
            i32.const 0
            i32.store
            local.get 2
            i32.eqz
            br_if 2 (;@2;)
            local.get 4
            i32.const 4
            i32.add
            call $_ZN10rayon_core8registry23default_global_registry17he1e6784285ac3562E
            block  ;; label = @5
              block  ;; label = @6
                local.get 4
                i32.load8_u offset=4
                local.tee 1
                i32.const 6
                i32.ne
                br_if 0 (;@6;)
                local.get 4
                local.get 4
                i32.load offset=8
                local.tee 3
                i32.store offset=28
                block  ;; label = @7
                  i32.const 0
                  i32.load offset=1059000
                  br_if 0 (;@7;)
                  i32.const 0
                  local.get 3
                  i32.store offset=1059000
                  i32.const 1059000
                  local.set 3
                  br 2 (;@5;)
                end
                local.get 3
                local.get 3
                i32.load
                local.tee 5
                i32.const -1
                i32.add
                i32.store
                block  ;; label = @7
                  local.get 5
                  i32.const 1
                  i32.ne
                  br_if 0 (;@7;)
                  local.get 4
                  i32.const 28
                  i32.add
                  call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h0960bc771bfafaf1E
                end
                i32.const 1059000
                local.set 3
                br 1 (;@5;)
              end
              local.get 4
              i32.const 30
              i32.add
              local.get 4
              i32.load8_u offset=7
              i32.store8
              local.get 4
              local.get 4
              i32.load16_u offset=5 align=1
              i32.store16 offset=28
              local.get 4
              i32.load offset=8
              local.set 3
            end
            local.get 2
            i32.load offset=4
            local.set 6
            block  ;; label = @5
              block  ;; label = @6
                local.get 2
                i32.load8_u
                local.tee 5
                i32.const 6
                i32.gt_u
                br_if 0 (;@6;)
                local.get 5
                i32.const 3
                i32.ne
                br_if 1 (;@5;)
              end
              local.get 6
              i32.load
              local.set 7
              block  ;; label = @6
                local.get 6
                i32.const 4
                i32.add
                i32.load
                local.tee 5
                i32.load
                local.tee 8
                i32.eqz
                br_if 0 (;@6;)
                local.get 7
                local.get 8
                call_indirect (type 0)
              end
              block  ;; label = @6
                local.get 5
                i32.load offset=4
                local.tee 8
                i32.eqz
                br_if 0 (;@6;)
                local.get 7
                local.get 8
                local.get 5
                i32.load offset=8
                call $__rust_dealloc
              end
              local.get 6
              i32.const 12
              i32.const 4
              call $__rust_dealloc
            end
            local.get 2
            local.get 1
            i32.store8
            local.get 2
            local.get 4
            i32.load16_u offset=28
            i32.store16 offset=1 align=1
            local.get 2
            local.get 3
            i32.store offset=4
            local.get 2
            i32.const 3
            i32.add
            local.get 4
            i32.const 30
            i32.add
            i32.load8_u
            i32.store8
            local.get 0
            i32.const 3
            i32.store8
          end
          local.get 4
          i32.const 32
          i32.add
          global.set $__stack_pointer
          return
        end
        local.get 4
        i32.const 0
        i32.store offset=20
        local.get 4
        i32.const 1
        i32.store offset=8
        local.get 4
        i32.const 1050384
        i32.store offset=4
        local.get 4
        i64.const 4
        i64.store offset=12 align=4
        local.get 4
        i32.const 4
        i32.add
        local.get 3
        call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
        unreachable
      end
      i32.const 1050532
      call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
      unreachable
    end
    local.get 4
    i32.const 0
    i32.store offset=20
    local.get 4
    i32.const 1
    i32.store offset=8
    local.get 4
    i32.const 1050448
    i32.store offset=4
    local.get 4
    i64.const 4
    i64.store offset=12 align=4
    local.get 4
    i32.const 4
    i32.add
    local.get 3
    call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
    unreachable)
  (func $_ZN4core9panicking13assert_failed17h9a3e3b4c3cfd5ab0E.llvm.6263538152696972293 (type 2) (param i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 1050588
    i32.store offset=12
    local.get 2
    local.get 0
    i32.store offset=8
    i32.const 0
    local.get 2
    i32.const 8
    i32.add
    i32.const 1050700
    local.get 2
    i32.const 12
    i32.add
    i32.const 1050700
    local.get 1
    i32.const 1050684
    call $_ZN4core9panicking19assert_failed_inner17he4920e028524a869E
    unreachable)
  (func $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h053a31c8c3f17766E (type 4) (param i32 i32) (result i32)
    local.get 0
    i32.load
    local.get 1
    call $_ZN43_$LT$bool$u20$as$u20$core..fmt..Display$GT$3fmt17h90c5085ed9b38d31E)
  (func $_ZN10rayon_core5sleep5Sleep5sleep17h61ecc6b84acce2aaE (type 12) (param i32 i32 i32 i32)
    (local i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 4
    global.set $__stack_pointer
    local.get 2
    local.get 2
    i32.load
    local.tee 5
    i32.const 1
    local.get 5
    select
    i32.store
    block  ;; label = @1
      local.get 5
      br_if 0 (;@1;)
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 1
                    i32.load
                    local.tee 5
                    local.get 0
                    i32.load offset=8
                    local.tee 6
                    i32.ge_u
                    br_if 0 (;@8;)
                    local.get 4
                    local.get 0
                    i32.load offset=4
                    local.get 5
                    i32.const 6
                    i32.shl
                    i32.add
                    local.tee 7
                    i32.load8_u
                    local.tee 5
                    i32.store8 offset=7
                    local.get 7
                    i32.const 1
                    i32.store8
                    local.get 5
                    i32.const 1
                    i32.eq
                    br_if 3 (;@5;)
                    local.get 2
                    i32.const 2
                    local.get 2
                    i32.load
                    local.tee 5
                    local.get 5
                    i32.const 1
                    i32.eq
                    select
                    i32.store
                    local.get 5
                    i32.const 1
                    i32.ne
                    br_if 1 (;@7;)
                    loop  ;; label = @9
                      local.get 0
                      i32.load offset=12
                      local.tee 5
                      i32.const 16
                      i32.shr_u
                      local.get 1
                      i32.load offset=8
                      i32.ne
                      br_if 3 (;@6;)
                      local.get 0
                      local.get 5
                      i32.const 1
                      i32.add
                      local.get 0
                      i32.load offset=12
                      local.tee 6
                      local.get 6
                      local.get 5
                      i32.eq
                      local.tee 5
                      select
                      i32.store offset=12
                      local.get 5
                      i32.eqz
                      br_if 0 (;@9;)
                    end
                    local.get 3
                    i32.load offset=160
                    local.tee 5
                    i32.load offset=132
                    local.get 5
                    i32.load offset=128
                    i32.sub
                    i32.const 0
                    i32.gt_s
                    br_if 4 (;@4;)
                    local.get 3
                    i32.load offset=140
                    local.tee 5
                    i32.load offset=128
                    local.get 5
                    i32.load offset=64
                    i32.xor
                    i32.const 1
                    i32.gt_u
                    br_if 4 (;@4;)
                    local.get 7
                    i32.const 1
                    i32.store8 offset=1
                    local.get 7
                    i32.const 2
                    i32.add
                    local.set 0
                    loop  ;; label = @9
                      local.get 0
                      local.get 7
                      call $_ZN3std3sys4sync7condvar10no_threads7Condvar4wait17hb91e5e8042bc81e5E
                      local.get 7
                      i32.load8_u offset=1
                      br_if 0 (;@9;)
                      br 6 (;@3;)
                    end
                  end
                  local.get 5
                  local.get 6
                  i32.const 1051044
                  call $_ZN4core9panicking18panic_bounds_check17h6c9fc24fb71a0cb6E
                  unreachable
                end
                local.get 1
                i64.const -4294967296
                i64.store offset=4 align=4
                br 4 (;@2;)
              end
              local.get 1
              i64.const -4294967264
              i64.store offset=4 align=4
              local.get 2
              i32.load
              i32.const 3
              i32.eq
              br_if 3 (;@2;)
              local.get 2
              i32.const 0
              local.get 2
              i32.load
              local.tee 0
              local.get 0
              i32.const 2
              i32.eq
              select
              i32.store
              br 3 (;@2;)
            end
            local.get 4
            i64.const 0
            i64.store offset=20 align=4
            local.get 4
            i64.const 17179869185
            i64.store offset=12 align=4
            local.get 4
            i32.const 1050580
            i32.store offset=8
            local.get 4
            i32.const 7
            i32.add
            local.get 4
            i32.const 8
            i32.add
            call $_ZN4core9panicking13assert_failed17h9a3e3b4c3cfd5ab0E.llvm.6263538152696972293
            unreachable
          end
          local.get 0
          local.get 0
          i32.load offset=12
          i32.const -1
          i32.add
          i32.store offset=12
        end
        local.get 1
        i64.const -4294967296
        i64.store offset=4 align=4
        local.get 2
        i32.load
        i32.const 3
        i32.eq
        br_if 0 (;@2;)
        local.get 2
        i32.const 0
        local.get 2
        i32.load
        local.tee 0
        local.get 0
        i32.const 2
        i32.eq
        select
        i32.store
      end
      local.get 7
      i32.const 0
      i32.store8
    end
    local.get 4
    i32.const 32
    i32.add
    global.set $__stack_pointer)
  (func $_ZN10rayon_core5sleep5Sleep16wake_any_threads17h17244bbca7ed393dE (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      local.get 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.load offset=8
      local.tee 3
      i32.eqz
      br_if 0 (;@1;)
      i32.const 0
      local.set 4
      i32.const 0
      local.set 5
      loop  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 0
            i32.load offset=8
            local.tee 6
            local.get 5
            i32.le_u
            br_if 0 (;@4;)
            local.get 2
            local.get 0
            i32.load offset=4
            local.get 4
            i32.add
            local.tee 6
            i32.load8_u
            local.tee 7
            i32.store8 offset=7
            local.get 6
            i32.const 1
            i32.store8
            block  ;; label = @5
              local.get 7
              i32.const 1
              i32.eq
              br_if 0 (;@5;)
              block  ;; label = @6
                local.get 6
                i32.const 1
                i32.add
                local.tee 7
                i32.load8_u
                br_if 0 (;@6;)
                local.get 6
                i32.const 0
                i32.store8
                br 3 (;@3;)
              end
              local.get 7
              i32.const 0
              i32.store8
              local.get 0
              local.get 0
              i32.load offset=12
              i32.const -1
              i32.add
              i32.store offset=12
              local.get 6
              i32.const 0
              i32.store8
              local.get 1
              i32.const -1
              i32.add
              local.tee 1
              br_if 2 (;@3;)
              br 4 (;@1;)
            end
            local.get 2
            i64.const 0
            i64.store offset=20 align=4
            local.get 2
            i64.const 17179869185
            i64.store offset=12 align=4
            local.get 2
            i32.const 1050580
            i32.store offset=8
            local.get 2
            i32.const 7
            i32.add
            local.get 2
            i32.const 8
            i32.add
            call $_ZN4core9panicking13assert_failed17h9a3e3b4c3cfd5ab0E.llvm.6263538152696972293
            unreachable
          end
          local.get 5
          local.get 6
          i32.const 1051060
          call $_ZN4core9panicking18panic_bounds_check17h6c9fc24fb71a0cb6E
          unreachable
        end
        local.get 4
        i32.const 64
        i32.add
        local.set 4
        local.get 3
        local.get 5
        i32.const 1
        i32.add
        local.tee 5
        i32.ne
        br_if 0 (;@2;)
      end
    end
    local.get 2
    i32.const 32
    i32.add
    global.set $__stack_pointer)
  (func $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$3pop17h1ca15d87b82f6609E (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    i32.const 0
    local.set 2
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.load
        local.tee 3
        i32.load offset=132
        local.tee 4
        local.get 3
        i32.load offset=128
        i32.sub
        local.tee 5
        i32.const 1
        i32.ge_s
        br_if 0 (;@2;)
        br 1 (;@1;)
      end
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 1
            i32.load8_u offset=12
            br_if 0 (;@4;)
            local.get 3
            local.get 3
            i32.load offset=128
            local.tee 6
            i32.const 1
            i32.add
            i32.store offset=128
            local.get 6
            local.get 4
            i32.sub
            i32.const -1
            i32.gt_s
            br_if 1 (;@3;)
            local.get 1
            i32.load offset=8
            local.tee 4
            i32.const 4
            i32.div_s
            local.set 7
            local.get 1
            i32.load offset=4
            local.get 4
            i32.const -1
            i32.add
            local.get 6
            i32.and
            i32.const 3
            i32.shl
            i32.add
            local.tee 2
            i32.load offset=4
            local.set 3
            local.get 2
            i32.load
            local.set 2
            local.get 4
            i32.const 65
            i32.lt_u
            br_if 3 (;@1;)
            local.get 5
            local.get 7
            i32.gt_s
            br_if 3 (;@1;)
            local.get 1
            local.get 4
            i32.const 1
            i32.shr_u
            call $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$6resize17hdbab0c832ae592e0E.llvm.13464497517775300485
            br 3 (;@1;)
          end
          local.get 3
          local.get 4
          i32.const -1
          i32.add
          local.tee 5
          i32.store offset=132
          i32.const 0
          local.set 2
          local.get 5
          local.get 1
          i32.load
          local.tee 6
          i32.load offset=128
          local.tee 7
          i32.sub
          local.tee 8
          i32.const 0
          i32.lt_s
          br_if 1 (;@2;)
          local.get 1
          i32.load offset=4
          local.get 1
          i32.load offset=8
          local.tee 9
          i32.const -1
          i32.add
          local.get 5
          i32.and
          i32.const 3
          i32.shl
          i32.add
          local.tee 2
          i32.load offset=4
          local.set 3
          local.get 2
          i32.load
          local.set 2
          block  ;; label = @4
            local.get 5
            local.get 7
            i32.ne
            br_if 0 (;@4;)
            local.get 6
            local.get 4
            local.get 6
            i32.load offset=128
            local.tee 7
            local.get 7
            local.get 5
            i32.eq
            local.tee 5
            select
            i32.store offset=128
            local.get 1
            i32.load
            local.get 4
            i32.store offset=132
            local.get 5
            br_if 3 (;@1;)
            i32.const 0
            local.set 2
            br 3 (;@1;)
          end
          local.get 9
          i32.const 4
          i32.div_s
          local.set 4
          local.get 9
          i32.const 65
          i32.lt_u
          br_if 2 (;@1;)
          local.get 8
          local.get 4
          i32.ge_s
          br_if 2 (;@1;)
          local.get 1
          local.get 9
          i32.const 1
          i32.shr_u
          call $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$6resize17hdbab0c832ae592e0E.llvm.13464497517775300485
          br 2 (;@1;)
        end
        local.get 1
        i32.load
        local.get 6
        i32.store offset=128
        br 1 (;@1;)
      end
      local.get 6
      local.get 4
      i32.store offset=132
    end
    local.get 0
    local.get 3
    i32.store offset=4
    local.get 0
    local.get 2
    i32.store)
  (func $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$6resize17hdbab0c832ae592e0E.llvm.13464497517775300485 (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 1
    i32.const 3
    i32.shl
    local.set 3
    i32.const 0
    local.set 4
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 1
          i32.const 536870911
          i32.gt_u
          br_if 0 (;@3;)
          local.get 3
          i32.const 2147483644
          i32.gt_u
          br_if 0 (;@3;)
          local.get 0
          i32.load offset=8
          local.set 5
          local.get 0
          i32.load offset=4
          local.set 6
          local.get 0
          i32.load
          local.tee 7
          i32.load offset=128
          local.set 8
          local.get 7
          i32.load offset=132
          local.set 9
          block  ;; label = @4
            block  ;; label = @5
              local.get 3
              br_if 0 (;@5;)
              i32.const 4
              local.set 7
              br 1 (;@4;)
            end
            i32.const 0
            i32.load8_u offset=1058985
            drop
            i32.const 4
            local.set 4
            local.get 3
            i32.const 4
            call $__rust_alloc
            local.tee 7
            i32.eqz
            br_if 1 (;@3;)
          end
          block  ;; label = @4
            local.get 8
            local.get 9
            i32.eq
            br_if 0 (;@4;)
            local.get 1
            i32.const -1
            i32.add
            local.set 4
            local.get 5
            i32.const -1
            i32.add
            local.set 5
            local.get 8
            local.set 3
            block  ;; label = @5
              local.get 9
              local.get 8
              i32.sub
              i32.const 3
              i32.and
              local.tee 10
              i32.eqz
              br_if 0 (;@5;)
              local.get 8
              local.set 3
              loop  ;; label = @6
                local.get 7
                local.get 3
                local.get 4
                i32.and
                i32.const 3
                i32.shl
                i32.add
                local.get 6
                local.get 3
                local.get 5
                i32.and
                i32.const 3
                i32.shl
                i32.add
                i64.load align=4
                i64.store align=4
                local.get 3
                i32.const 1
                i32.add
                local.set 3
                local.get 10
                i32.const -1
                i32.add
                local.tee 10
                br_if 0 (;@6;)
              end
            end
            local.get 8
            local.get 9
            i32.sub
            i32.const -4
            i32.gt_u
            br_if 0 (;@4;)
            loop  ;; label = @5
              local.get 7
              local.get 3
              local.get 4
              i32.and
              i32.const 3
              i32.shl
              i32.add
              local.get 6
              local.get 3
              local.get 5
              i32.and
              i32.const 3
              i32.shl
              i32.add
              i64.load align=4
              i64.store align=4
              local.get 7
              local.get 3
              i32.const 1
              i32.add
              local.tee 10
              local.get 4
              i32.and
              i32.const 3
              i32.shl
              i32.add
              local.get 6
              local.get 10
              local.get 5
              i32.and
              i32.const 3
              i32.shl
              i32.add
              i64.load align=4
              i64.store align=4
              local.get 7
              local.get 3
              i32.const 2
              i32.add
              local.tee 10
              local.get 4
              i32.and
              i32.const 3
              i32.shl
              i32.add
              local.get 6
              local.get 10
              local.get 5
              i32.and
              i32.const 3
              i32.shl
              i32.add
              i64.load align=4
              i64.store align=4
              local.get 7
              local.get 3
              i32.const 3
              i32.add
              local.tee 10
              local.get 4
              i32.and
              i32.const 3
              i32.shl
              i32.add
              local.get 6
              local.get 10
              local.get 5
              i32.and
              i32.const 3
              i32.shl
              i32.add
              i64.load align=4
              i64.store align=4
              local.get 3
              i32.const 4
              i32.add
              local.tee 3
              local.get 9
              i32.ne
              br_if 0 (;@5;)
            end
          end
          i32.const 1059020
          local.set 3
          block  ;; label = @4
            i32.const 0
            i32.load offset=1059016
            br_if 0 (;@4;)
            i32.const 1059016
            i32.const 0
            call $_ZN3std3sys12thread_local6statik20LazyStorage$LT$T$GT$10initialize17heb39f68f3dc1c612E
            local.set 3
          end
          local.get 2
          local.get 3
          i32.load
          local.tee 3
          i32.store offset=16
          local.get 3
          i32.load offset=1036
          local.tee 6
          i32.const -1
          i32.eq
          br_if 1 (;@2;)
          local.get 3
          local.get 6
          i32.const 1
          i32.add
          i32.store offset=1036
          block  ;; label = @4
            local.get 6
            br_if 0 (;@4;)
            local.get 3
            i32.load offset=4
            i32.load offset=192
            local.set 6
            local.get 3
            local.get 3
            i32.load offset=1044
            local.tee 4
            i32.const 1
            i32.add
            i32.store offset=1044
            local.get 3
            local.get 6
            i32.const 1
            i32.or
            i32.store offset=1088
            local.get 4
            i32.const 127
            i32.and
            br_if 0 (;@4;)
            local.get 3
            i32.load offset=4
            i32.const 64
            i32.add
            local.get 2
            i32.const 16
            i32.add
            call $_ZN15crossbeam_epoch8internal6Global7collect17h7d10bdb777c95448E
          end
          local.get 2
          i32.load offset=16
          local.set 3
          local.get 0
          local.get 1
          i32.store offset=8
          local.get 0
          local.get 7
          i32.store offset=4
          local.get 2
          local.get 3
          i32.store offset=12
          local.get 0
          i32.load
          local.set 6
          i32.const 0
          i32.load8_u offset=1058985
          drop
          i32.const 8
          i32.const 4
          call $__rust_alloc
          local.tee 3
          i32.eqz
          br_if 2 (;@1;)
          local.get 3
          local.get 1
          i32.store offset=4
          local.get 3
          local.get 7
          i32.store
          local.get 6
          i32.load offset=64
          local.set 7
          local.get 6
          local.get 3
          i32.store offset=64
          block  ;; label = @4
            block  ;; label = @5
              local.get 2
              i32.load offset=12
              local.tee 3
              br_if 0 (;@5;)
              block  ;; label = @6
                local.get 7
                i32.const -4
                i32.and
                local.tee 3
                i32.load offset=4
                local.tee 7
                i32.eqz
                br_if 0 (;@6;)
                local.get 3
                i32.load
                local.get 7
                i32.const 3
                i32.shl
                i32.const 4
                call $__rust_dealloc
              end
              local.get 3
              i32.const 8
              i32.const 4
              call $__rust_dealloc
              br 1 (;@4;)
            end
            local.get 2
            local.get 7
            i32.store offset=20
            local.get 2
            i32.const 14
            i32.store offset=16
            local.get 3
            local.get 2
            i32.const 16
            i32.add
            local.get 2
            i32.const 12
            i32.add
            call $_ZN15crossbeam_epoch8internal5Local5defer17h86d466552d8e8645E
          end
          block  ;; label = @4
            local.get 1
            i32.const 128
            i32.lt_u
            br_if 0 (;@4;)
            local.get 2
            i32.const 12
            i32.add
            call $_ZN15crossbeam_epoch5guard5Guard5flush17h32b4d49dc69f4d8aE
          end
          block  ;; label = @4
            local.get 2
            i32.load offset=12
            local.tee 3
            i32.eqz
            br_if 0 (;@4;)
            local.get 3
            local.get 3
            i32.load offset=1036
            local.tee 7
            i32.const -1
            i32.add
            i32.store offset=1036
            local.get 7
            i32.const 1
            i32.ne
            br_if 0 (;@4;)
            local.get 3
            i32.const 0
            i32.store offset=1088
            local.get 3
            i32.load offset=1040
            br_if 0 (;@4;)
            local.get 3
            call $_ZN15crossbeam_epoch8internal5Local8finalize17ha822a731c89268d9E
          end
          local.get 2
          i32.const 32
          i32.add
          global.set $__stack_pointer
          return
        end
        local.get 4
        local.get 3
        i32.const 1051536
        call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
        unreachable
      end
      i32.const 1051304
      call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
      unreachable
    end
    i32.const 4
    i32.const 8
    call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
    unreachable)
  (func $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$8new_fifo17had5b41e2b64bb3a1E (type 0) (param i32)
    (local i32 i32 i32)
    i32.const 0
    i32.load8_u offset=1058985
    drop
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          i32.const 512
          i32.const 4
          call $__rust_alloc
          local.tee 1
          i32.eqz
          br_if 0 (;@3;)
          i32.const 0
          i32.load8_u offset=1058985
          drop
          i32.const 8
          i32.const 4
          call $__rust_alloc
          local.tee 2
          i32.eqz
          br_if 1 (;@2;)
          local.get 2
          i32.const 64
          i32.store offset=4
          local.get 2
          local.get 1
          i32.store
          i32.const 0
          i32.load8_u offset=1058985
          drop
          i32.const 192
          i32.const 64
          call $__rust_alloc
          local.tee 3
          br_if 2 (;@1;)
          i32.const 64
          i32.const 192
          call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
          unreachable
        end
        i32.const 4
        i32.const 512
        i32.const 1051536
        call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
        unreachable
      end
      i32.const 4
      i32.const 8
      call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
      unreachable
    end
    local.get 3
    i64.const 0
    i64.store offset=128
    local.get 3
    local.get 2
    i32.store offset=64
    local.get 3
    i64.const 4294967297
    i64.store
    local.get 0
    i32.const 0
    i32.store8 offset=12
    local.get 0
    i32.const 64
    i32.store offset=8
    local.get 0
    local.get 1
    i32.store offset=4
    local.get 0
    local.get 3
    i32.store)
  (func $_ZN15crossbeam_deque5deque15Worker$LT$T$GT$8new_lifo17hae9d1ea4214d0b5bE (type 0) (param i32)
    (local i32 i32 i32)
    i32.const 0
    i32.load8_u offset=1058985
    drop
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          i32.const 512
          i32.const 4
          call $__rust_alloc
          local.tee 1
          i32.eqz
          br_if 0 (;@3;)
          i32.const 0
          i32.load8_u offset=1058985
          drop
          i32.const 8
          i32.const 4
          call $__rust_alloc
          local.tee 2
          i32.eqz
          br_if 1 (;@2;)
          local.get 2
          i32.const 64
          i32.store offset=4
          local.get 2
          local.get 1
          i32.store
          i32.const 0
          i32.load8_u offset=1058985
          drop
          i32.const 192
          i32.const 64
          call $__rust_alloc
          local.tee 3
          br_if 2 (;@1;)
          i32.const 64
          i32.const 192
          call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
          unreachable
        end
        i32.const 4
        i32.const 512
        i32.const 1051536
        call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
        unreachable
      end
      i32.const 4
      i32.const 8
      call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
      unreachable
    end
    local.get 3
    i64.const 0
    i64.store offset=128
    local.get 3
    local.get 2
    i32.store offset=64
    local.get 3
    i64.const 4294967297
    i64.store
    local.get 0
    i32.const 1
    i32.store8 offset=12
    local.get 0
    i32.const 64
    i32.store offset=8
    local.get 0
    local.get 1
    i32.store offset=4
    local.get 0
    local.get 3
    i32.store)
  (func $_ZN15crossbeam_deque5deque16Stealer$LT$T$GT$5steal17hb6210202028f7ff8E (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 1
    i32.load
    local.tee 3
    i32.load offset=128
    local.set 4
    block  ;; label = @1
      i32.const 0
      i32.load offset=1059016
      br_if 0 (;@1;)
      i32.const 1059016
      i32.const 0
      call $_ZN3std3sys12thread_local6statik20LazyStorage$LT$T$GT$10initialize17heb39f68f3dc1c612E
      drop
    end
    i32.const 1059020
    local.set 1
    block  ;; label = @1
      i32.const 0
      i32.load offset=1059016
      br_if 0 (;@1;)
      i32.const 1059016
      i32.const 0
      call $_ZN3std3sys12thread_local6statik20LazyStorage$LT$T$GT$10initialize17heb39f68f3dc1c612E
      local.set 1
    end
    local.get 2
    local.get 1
    i32.load
    local.tee 1
    i32.store offset=12
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 1
            i32.load offset=1036
            local.tee 5
            i32.const -1
            i32.eq
            br_if 0 (;@4;)
            local.get 1
            local.get 5
            i32.const 1
            i32.add
            i32.store offset=1036
            block  ;; label = @5
              local.get 5
              br_if 0 (;@5;)
              local.get 1
              i32.load offset=4
              i32.load offset=192
              local.set 5
              local.get 1
              local.get 1
              i32.load offset=1044
              local.tee 6
              i32.const 1
              i32.add
              i32.store offset=1044
              local.get 1
              local.get 5
              i32.const 1
              i32.or
              i32.store offset=1088
              local.get 6
              i32.const 127
              i32.and
              br_if 0 (;@5;)
              local.get 1
              i32.load offset=4
              i32.const 64
              i32.add
              local.get 2
              i32.const 12
              i32.add
              call $_ZN15crossbeam_epoch8internal6Global7collect17h7d10bdb777c95448E
            end
            local.get 2
            i32.load offset=12
            local.set 1
            block  ;; label = @5
              local.get 3
              i32.load offset=132
              local.get 4
              i32.sub
              i32.const 1
              i32.ge_s
              br_if 0 (;@5;)
              i32.const 0
              local.set 5
              br 2 (;@3;)
            end
            local.get 3
            i32.load offset=64
            local.tee 6
            i32.const -4
            i32.and
            local.tee 5
            i32.load
            local.get 5
            i32.load offset=4
            i32.const -1
            i32.add
            local.get 4
            i32.and
            i32.const 3
            i32.shl
            i32.add
            local.tee 5
            i32.load offset=4
            local.set 7
            local.get 5
            i32.load
            local.set 8
            i32.const 2
            local.set 5
            local.get 6
            local.get 3
            i32.load offset=64
            i32.ne
            br_if 1 (;@3;)
            local.get 3
            local.get 4
            i32.const 1
            i32.add
            local.get 3
            i32.load offset=128
            local.tee 6
            local.get 6
            local.get 4
            i32.eq
            select
            i32.store offset=128
            local.get 6
            local.get 4
            i32.ne
            br_if 1 (;@3;)
            local.get 0
            local.get 7
            i32.store offset=8
            local.get 0
            local.get 8
            i32.store offset=4
            local.get 0
            i32.const 1
            i32.store
            local.get 1
            i32.eqz
            br_if 3 (;@1;)
            local.get 1
            local.get 1
            i32.load offset=1036
            local.tee 3
            i32.const -1
            i32.add
            i32.store offset=1036
            local.get 3
            i32.const 1
            i32.ne
            br_if 3 (;@1;)
            local.get 1
            i32.const 0
            i32.store offset=1088
            local.get 1
            i32.load offset=1040
            br_if 3 (;@1;)
            br 2 (;@2;)
          end
          i32.const 1051304
          call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
          unreachable
        end
        local.get 0
        local.get 5
        i32.store
        local.get 1
        i32.eqz
        br_if 1 (;@1;)
        local.get 1
        local.get 1
        i32.load offset=1036
        local.tee 3
        i32.const -1
        i32.add
        i32.store offset=1036
        local.get 3
        i32.const 1
        i32.ne
        br_if 1 (;@1;)
        local.get 1
        i32.const 0
        i32.store offset=1088
        local.get 1
        i32.load offset=1040
        br_if 1 (;@1;)
      end
      local.get 1
      call $_ZN15crossbeam_epoch8internal5Local8finalize17ha822a731c89268d9E
    end
    local.get 2
    i32.const 16
    i32.add
    global.set $__stack_pointer)
  (func $_ZN15crossbeam_deque5deque17Injector$LT$T$GT$4push17h9e2e7a20ad51dc77E (type 3) (param i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32)
    local.get 0
    i32.load offset=68
    local.set 3
    local.get 0
    i32.load offset=64
    local.set 4
    i32.const 0
    local.set 5
    i32.const 0
    local.set 6
    block  ;; label = @1
      block  ;; label = @2
        loop  ;; label = @3
          block  ;; label = @4
            local.get 4
            i32.const 1
            i32.shr_u
            i32.const 63
            i32.and
            local.tee 7
            i32.const 63
            i32.ne
            br_if 0 (;@4;)
            loop  ;; label = @5
              block  ;; label = @6
                local.get 6
                i32.const 6
                i32.gt_u
                br_if 0 (;@6;)
                i32.const 0
                local.set 4
                loop  ;; label = @7
                  local.get 4
                  local.get 6
                  i32.shr_u
                  local.set 7
                  local.get 4
                  i32.const 1
                  i32.add
                  local.set 4
                  local.get 7
                  i32.eqz
                  br_if 0 (;@7;)
                end
              end
              local.get 6
              local.get 6
              i32.const 11
              i32.lt_u
              i32.add
              local.set 6
              local.get 0
              i32.load offset=64
              local.tee 4
              i32.const 1
              i32.shr_u
              i32.const 63
              i32.and
              local.tee 7
              i32.const 63
              i32.eq
              br_if 0 (;@5;)
            end
            local.get 0
            i32.load offset=68
            local.set 3
          end
          block  ;; label = @4
            local.get 7
            i32.const 62
            i32.ne
            br_if 0 (;@4;)
            local.get 5
            br_if 0 (;@4;)
            i32.const 0
            i32.load8_u offset=1058985
            drop
            i32.const 760
            i32.const 4
            call $__rust_alloc_zeroed
            local.tee 5
            br_if 0 (;@4;)
            i32.const 4
            i32.const 760
            call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
            unreachable
          end
          local.get 0
          local.get 4
          i32.const 2
          i32.add
          local.get 0
          i32.load offset=64
          local.tee 8
          local.get 8
          local.get 4
          i32.eq
          local.tee 9
          select
          i32.store offset=64
          block  ;; label = @4
            block  ;; label = @5
              local.get 9
              i32.eqz
              br_if 0 (;@5;)
              local.get 7
              i32.const 62
              i32.eq
              br_if 1 (;@4;)
              local.get 3
              local.get 7
              i32.const 12
              i32.mul
              i32.add
              local.tee 4
              i32.const 8
              i32.add
              local.get 2
              i32.store
              local.get 4
              i32.const 4
              i32.add
              local.get 1
              i32.store
              local.get 4
              i32.const 12
              i32.add
              local.tee 4
              local.get 4
              i32.load
              i32.const 1
              i32.or
              i32.store
              local.get 5
              i32.eqz
              br_if 3 (;@2;)
              local.get 5
              i32.const 760
              i32.const 4
              call $__rust_dealloc
              return
            end
            local.get 6
            local.get 6
            i32.const 7
            i32.lt_u
            i32.add
            local.set 6
            local.get 0
            i32.load offset=68
            local.set 3
            local.get 8
            local.set 4
            br 1 (;@3;)
          end
        end
        local.get 5
        i32.eqz
        br_if 1 (;@1;)
        local.get 0
        local.get 5
        i32.store offset=68
        local.get 0
        local.get 4
        i32.const 4
        i32.add
        i32.store offset=64
        local.get 3
        local.get 2
        i32.store offset=752
        local.get 3
        local.get 1
        i32.store offset=748
        local.get 3
        local.get 5
        i32.store
        local.get 3
        local.get 3
        i32.load offset=756
        i32.const 1
        i32.or
        i32.store offset=756
      end
      return
    end
    i32.const 1051180
    call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
    unreachable)
  (func $_ZN15crossbeam_deque5deque17Injector$LT$T$GT$5steal17h460d132b966242adE (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32)
    block  ;; label = @1
      local.get 1
      i32.load
      local.tee 2
      i32.const 1
      i32.shr_u
      local.tee 3
      i32.const 63
      i32.and
      local.tee 4
      i32.const 63
      i32.ne
      br_if 0 (;@1;)
      i32.const 0
      local.set 5
      loop  ;; label = @2
        block  ;; label = @3
          local.get 5
          i32.const 6
          i32.gt_u
          br_if 0 (;@3;)
          i32.const 0
          local.set 2
          loop  ;; label = @4
            local.get 2
            local.get 5
            i32.shr_u
            local.set 3
            local.get 2
            i32.const 1
            i32.add
            local.set 2
            local.get 3
            i32.eqz
            br_if 0 (;@4;)
          end
        end
        local.get 5
        local.get 5
        i32.const 11
        i32.lt_u
        i32.add
        local.set 5
        local.get 1
        i32.load
        local.tee 2
        i32.const 1
        i32.shr_u
        local.tee 3
        i32.const 63
        i32.and
        local.tee 4
        i32.const 63
        i32.eq
        br_if 0 (;@2;)
      end
    end
    local.get 1
    i32.load offset=4
    local.set 6
    local.get 2
    i32.const 2
    i32.add
    local.set 7
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        i32.const 1
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        local.set 5
        local.get 3
        local.get 1
        i32.load offset=64
        local.tee 8
        i32.const 1
        i32.shr_u
        i32.eq
        br_if 1 (;@1;)
        local.get 7
        local.get 8
        local.get 2
        i32.xor
        i32.const 127
        i32.gt_u
        i32.or
        local.set 7
      end
      local.get 1
      local.get 7
      local.get 1
      i32.load
      local.tee 3
      local.get 3
      local.get 2
      i32.eq
      select
      i32.store
      i32.const 2
      local.set 5
      local.get 3
      local.get 2
      i32.ne
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 4
        i32.const 62
        i32.ne
        br_if 0 (;@2;)
        block  ;; label = @3
          local.get 6
          i32.load
          local.tee 2
          br_if 0 (;@3;)
          i32.const 0
          local.set 5
          loop  ;; label = @4
            block  ;; label = @5
              local.get 5
              i32.const 6
              i32.gt_u
              br_if 0 (;@5;)
              i32.const 0
              local.set 2
              loop  ;; label = @6
                local.get 2
                local.get 5
                i32.shr_u
                local.set 3
                local.get 2
                i32.const 1
                i32.add
                local.set 2
                local.get 3
                i32.eqz
                br_if 0 (;@6;)
              end
            end
            local.get 5
            local.get 5
            i32.const 11
            i32.lt_u
            i32.add
            local.set 5
            local.get 6
            i32.load
            local.tee 2
            i32.eqz
            br_if 0 (;@4;)
          end
        end
        local.get 2
        i32.load
        local.set 3
        local.get 1
        local.get 2
        i32.store offset=4
        local.get 1
        local.get 7
        i32.const -2
        i32.and
        local.get 3
        i32.const 0
        i32.ne
        i32.or
        i32.const 2
        i32.add
        i32.store
      end
      local.get 6
      local.get 4
      i32.const 12
      i32.mul
      i32.add
      local.tee 2
      i32.const 4
      i32.add
      local.set 1
      block  ;; label = @2
        local.get 2
        i32.const 12
        i32.add
        i32.load8_u
        i32.const 1
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        local.set 5
        loop  ;; label = @3
          block  ;; label = @4
            local.get 5
            i32.const 6
            i32.gt_u
            br_if 0 (;@4;)
            i32.const 0
            local.set 2
            loop  ;; label = @5
              local.get 2
              local.get 5
              i32.shr_u
              local.set 3
              local.get 2
              i32.const 1
              i32.add
              local.set 2
              local.get 3
              i32.eqz
              br_if 0 (;@5;)
            end
          end
          local.get 5
          local.get 5
          i32.const 11
          i32.lt_u
          i32.add
          local.set 5
          local.get 1
          i32.load8_u offset=8
          i32.const 1
          i32.and
          i32.eqz
          br_if 0 (;@3;)
        end
      end
      local.get 1
      i32.load offset=4
      local.set 5
      local.get 1
      i32.load
      local.set 7
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 4
            i32.const 62
            i32.eq
            br_if 0 (;@4;)
            local.get 1
            local.get 1
            i32.load offset=8
            local.tee 2
            i32.const 2
            i32.or
            i32.store offset=8
            local.get 2
            i32.const 4
            i32.and
            i32.eqz
            br_if 2 (;@2;)
            local.get 4
            i32.eqz
            br_if 1 (;@3;)
          end
          local.get 6
          local.get 4
          i32.const 12
          i32.mul
          i32.add
          local.set 2
          loop  ;; label = @4
            block  ;; label = @5
              local.get 2
              i32.load8_u
              i32.const 2
              i32.and
              br_if 0 (;@5;)
              local.get 2
              local.get 2
              i32.load
              local.tee 3
              i32.const 4
              i32.or
              i32.store
              local.get 3
              i32.const 2
              i32.and
              i32.eqz
              br_if 3 (;@2;)
            end
            local.get 2
            i32.const -12
            i32.add
            local.set 2
            local.get 4
            i32.const -1
            i32.add
            local.tee 4
            br_if 0 (;@4;)
          end
        end
        local.get 6
        i32.const 760
        i32.const 4
        call $__rust_dealloc
      end
      local.get 0
      local.get 5
      i32.store offset=8
      local.get 0
      local.get 7
      i32.store offset=4
      i32.const 1
      local.set 5
    end
    local.get 0
    local.get 5
    i32.store)
  (func $_ZN3std6thread7Builder16spawn_unchecked_17h8de3712b4d76d82fE (type 12) (param i32 i32 i32 i32)
    (local i32 i32 i32 i32 i32 i64 i64)
    global.get $__stack_pointer
    i32.const 64
    i32.sub
    local.tee 4
    global.set $__stack_pointer
    local.get 1
    i32.load offset=8
    local.set 5
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.load
        br_if 0 (;@2;)
        block  ;; label = @3
          i32.const 0
          i32.load offset=1059028
          local.tee 6
          br_if 0 (;@3;)
          i32.const 0
          i32.const 65537
          i32.store offset=1059028
          i32.const 65536
          local.set 7
          br 2 (;@1;)
        end
        local.get 6
        i32.const -1
        i32.add
        local.set 7
        br 1 (;@1;)
      end
      local.get 1
      i32.load offset=4
      local.set 7
    end
    local.get 1
    i32.load8_u offset=20
    local.set 8
    local.get 1
    i64.load offset=12 align=4
    local.set 9
    call $_ZN3std6thread8ThreadId3new17hb3378d65506352f2E
    local.set 10
    block  ;; label = @1
      block  ;; label = @2
        local.get 5
        i32.const -2147483648
        i32.ne
        br_if 0 (;@2;)
        local.get 4
        i32.const 8
        i32.add
        local.get 10
        call $_ZN3std6thread6Thread11new_unnamed17h5a535a66c682809cE
        local.get 4
        i32.load offset=12
        local.set 6
        local.get 4
        i32.load offset=8
        local.set 1
        br 1 (;@1;)
      end
      local.get 4
      local.get 9
      i64.store offset=36 align=4
      local.get 4
      local.get 5
      i32.store offset=32
      local.get 4
      i32.const 16
      i32.add
      local.get 10
      local.get 4
      i32.const 32
      i32.add
      call $_ZN3std6thread6Thread3new17hc7b0b42432d00df2E
      local.get 4
      i32.load offset=20
      local.set 6
      local.get 4
      i32.load offset=16
      local.set 1
    end
    local.get 4
    local.get 1
    i32.store offset=24
    local.get 4
    local.get 6
    i32.store offset=28
    block  ;; label = @1
      block  ;; label = @2
        local.get 8
        i32.const 1
        i32.and
        br_if 0 (;@2;)
        local.get 4
        i32.const 44
        i32.add
        local.get 4
        i32.const 24
        i32.add
        call $_ZN3std6thread9spawnhook15run_spawn_hooks17h0555442308226df3E
        local.get 4
        i32.load offset=28
        local.set 6
        local.get 4
        i32.load offset=24
        local.set 1
        br 1 (;@1;)
      end
      local.get 4
      i64.const 17179869184
      i64.store offset=44 align=4
      local.get 4
      i64.const 0
      i64.store offset=52 align=4
    end
    i32.const 0
    local.set 8
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 1
            i32.const 1
            i32.and
            i32.eqz
            br_if 0 (;@4;)
            local.get 6
            local.get 6
            i32.load
            local.tee 1
            i32.const 1
            i32.add
            i32.store
            i32.const 1
            local.set 8
            local.get 1
            i32.const -1
            i32.le_s
            br_if 1 (;@3;)
          end
          i32.const 0
          i32.load8_u offset=1058985
          drop
          i32.const 24
          i32.const 4
          call $__rust_alloc
          local.tee 5
          i32.eqz
          br_if 1 (;@2;)
          local.get 5
          i64.const 4294967297
          i64.store align=4
          local.get 5
          i32.const 0
          i32.store offset=12
          local.get 5
          local.get 3
          i32.store offset=8
          local.get 5
          i32.const 2
          i32.store
          local.get 4
          local.get 5
          i32.store offset=60
          i32.const 0
          br_if 0 (;@3;)
          block  ;; label = @4
            local.get 3
            i32.eqz
            br_if 0 (;@4;)
            local.get 3
            i32.const 8
            i32.add
            call $_ZN3std6thread6scoped9ScopeData29increment_num_running_threads17h1759ddd5d0975e85E
          end
          i32.const 0
          i32.load8_u offset=1058985
          drop
          i32.const 80
          i32.const 4
          call $__rust_alloc
          local.tee 1
          i32.eqz
          br_if 2 (;@1;)
          local.get 1
          local.get 6
          i32.store offset=4
          local.get 1
          local.get 8
          i32.store
          local.get 1
          local.get 4
          i64.load offset=44 align=4
          i64.store offset=8 align=4
          local.get 1
          local.get 5
          i32.store offset=24
          local.get 1
          local.get 2
          i64.load align=4
          i64.store offset=28 align=4
          local.get 1
          i32.const 16
          i32.add
          local.get 4
          i32.const 44
          i32.add
          i32.const 8
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 36
          i32.add
          local.get 2
          i32.const 8
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 44
          i32.add
          local.get 2
          i32.const 16
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 52
          i32.add
          local.get 2
          i32.const 24
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 60
          i32.add
          local.get 2
          i32.const 32
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 68
          i32.add
          local.get 2
          i32.const 40
          i32.add
          i64.load align=4
          i64.store align=4
          local.get 1
          i32.const 76
          i32.add
          local.get 2
          i32.const 48
          i32.add
          i32.load
          i32.store
          local.get 0
          i32.const 4
          i32.add
          local.get 7
          local.get 1
          i32.const 1051432
          call $_ZN3std3sys3pal4wasm6thread6Thread3new17h7a2d93e59351b2efE
          local.get 0
          i32.const 2
          i32.store
          local.get 5
          local.get 5
          i32.load
          local.tee 1
          i32.const -1
          i32.add
          i32.store
          block  ;; label = @4
            local.get 1
            i32.const 1
            i32.ne
            br_if 0 (;@4;)
            local.get 4
            i32.const 60
            i32.add
            call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h5a9d89913051a5edE
          end
          block  ;; label = @4
            local.get 4
            i32.load offset=24
            i32.eqz
            br_if 0 (;@4;)
            local.get 4
            i32.load offset=28
            local.tee 1
            local.get 1
            i32.load
            local.tee 1
            i32.const -1
            i32.add
            i32.store
            local.get 1
            i32.const 1
            i32.ne
            br_if 0 (;@4;)
            local.get 4
            i32.const 28
            i32.add
            call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h00ddb978cc2c31d9E
          end
          local.get 4
          i32.const 64
          i32.add
          global.set $__stack_pointer
          return
        end
        unreachable
      end
      i32.const 4
      i32.const 24
      call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
      unreachable
    end
    i32.const 4
    i32.const 80
    call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
    unreachable)
  (func $_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h44a1b090b5e82e47E (type 0) (param i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 144
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 0
    i32.load offset=4
    local.set 2
    i32.const 0
    local.set 3
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 0
          i32.load
          i32.const 1
          i32.ne
          br_if 0 (;@3;)
          local.get 2
          local.get 2
          i32.load
          local.tee 4
          i32.const 1
          i32.add
          i32.store
          i32.const 1
          local.set 3
          local.get 4
          i32.const -1
          i32.le_s
          br_if 1 (;@2;)
        end
        local.get 1
        i32.const 8
        i32.add
        local.get 3
        local.get 2
        call $_ZN3std6thread7current11set_current17h37b1d56a4498a5f9E
        local.get 1
        i32.load offset=8
        i32.const 2
        i32.ne
        br_if 1 (;@1;)
        local.get 1
        local.get 0
        call $_ZN3std6thread6Thread5cname17h8efee6976efe2f09E
        local.get 1
        i32.const 16
        i32.add
        i32.const 8
        i32.add
        local.tee 2
        local.get 0
        i32.const 36
        i32.add
        i64.load align=4
        i64.store
        local.get 1
        i32.const 16
        i32.add
        i32.const 16
        i32.add
        local.tee 3
        local.get 0
        i32.const 44
        i32.add
        i64.load align=4
        i64.store
        local.get 1
        i32.const 16
        i32.add
        i32.const 24
        i32.add
        local.tee 4
        local.get 0
        i32.const 52
        i32.add
        i64.load align=4
        i64.store
        local.get 1
        i32.const 16
        i32.add
        i32.const 32
        i32.add
        local.tee 5
        local.get 0
        i32.const 60
        i32.add
        i64.load align=4
        i64.store
        local.get 1
        i32.const 16
        i32.add
        i32.const 40
        i32.add
        local.tee 6
        local.get 0
        i32.const 68
        i32.add
        i64.load align=4
        i64.store
        local.get 1
        i32.const 16
        i32.add
        i32.const 48
        i32.add
        local.tee 7
        local.get 0
        i32.const 76
        i32.add
        i32.load
        i32.store
        local.get 1
        i32.const 16
        i32.add
        i32.const 60
        i32.add
        local.get 0
        i32.const 16
        i32.add
        local.tee 8
        i64.load align=4
        i64.store align=4
        local.get 1
        local.get 0
        i64.load offset=28 align=4
        i64.store offset=16
        local.get 1
        local.get 0
        i64.load offset=8 align=4
        i64.store offset=68 align=4
        local.get 1
        i32.const 88
        i32.add
        i32.const 8
        i32.add
        local.tee 9
        local.get 8
        i64.load align=4
        i64.store
        local.get 1
        local.get 0
        i64.load offset=8 align=4
        i64.store offset=88
        local.get 1
        i32.const 88
        i32.add
        call $_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h83c559db23fa37afE
        local.get 1
        i32.const 88
        i32.add
        i32.const 48
        i32.add
        local.get 7
        i32.load
        i32.store
        local.get 1
        i32.const 88
        i32.add
        i32.const 40
        i32.add
        local.get 6
        i64.load
        i64.store
        local.get 1
        i32.const 88
        i32.add
        i32.const 32
        i32.add
        local.get 5
        i64.load
        i64.store
        local.get 1
        i32.const 88
        i32.add
        i32.const 24
        i32.add
        local.get 4
        i64.load
        i64.store
        local.get 1
        i32.const 88
        i32.add
        i32.const 16
        i32.add
        local.get 3
        i64.load
        i64.store
        local.get 9
        local.get 2
        i64.load
        i64.store
        local.get 1
        local.get 1
        i64.load offset=16
        i64.store offset=88
        local.get 1
        i32.const 88
        i32.add
        call $_ZN3std3sys9backtrace28__rust_begin_short_backtrace17he53b213f08fa5222E
        block  ;; label = @3
          local.get 0
          i32.load offset=24
          local.tee 2
          i32.load offset=12
          i32.eqz
          br_if 0 (;@3;)
          local.get 2
          i32.load offset=16
          local.tee 3
          i32.eqz
          br_if 0 (;@3;)
          block  ;; label = @4
            local.get 2
            i32.load offset=20
            local.tee 4
            i32.load
            local.tee 5
            i32.eqz
            br_if 0 (;@4;)
            local.get 3
            local.get 5
            call_indirect (type 0)
          end
          local.get 4
          i32.load offset=4
          local.tee 5
          i32.eqz
          br_if 0 (;@3;)
          local.get 3
          local.get 5
          local.get 4
          i32.load offset=8
          call $__rust_dealloc
        end
        local.get 2
        i64.const 1
        i64.store offset=12 align=4
        local.get 1
        local.get 0
        i32.load offset=24
        local.tee 2
        i32.store offset=88
        local.get 2
        local.get 2
        i32.load
        local.tee 3
        i32.const -1
        i32.add
        i32.store
        block  ;; label = @3
          local.get 3
          i32.const 1
          i32.ne
          br_if 0 (;@3;)
          local.get 1
          i32.const 88
          i32.add
          call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h5a9d89913051a5edE
        end
        block  ;; label = @3
          local.get 0
          i32.load
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          i32.const 4
          i32.add
          local.tee 2
          i32.load
          local.tee 0
          local.get 0
          i32.load
          local.tee 0
          i32.const -1
          i32.add
          i32.store
          local.get 0
          i32.const 1
          i32.ne
          br_if 0 (;@3;)
          local.get 2
          call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h00ddb978cc2c31d9E
        end
        local.get 1
        i32.const 144
        i32.add
        global.set $__stack_pointer
        return
      end
      unreachable
    end
    call $_ZN3std3sys3pal4wasm6common14abort_internal17h968250bab15ff6b6E
    unreachable)
  (func $_ZN4core3ptr230drop_in_place$LT$std..thread..Builder..spawn_unchecked_$LT$$LT$rayon_core..registry..DefaultSpawn$u20$as$u20$rayon_core..registry..ThreadSpawn$GT$..spawn..$u7b$$u7b$closure$u7d$$u7d$$C$$LP$$RP$$GT$..$u7b$$u7b$closure$u7d$$u7d$$GT$17hc755cde14436e666E (type 0) (param i32)
    (local i32 i32 i32 i32 i32)
    block  ;; label = @1
      local.get 0
      i32.load
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.load offset=4
      local.tee 1
      local.get 1
      i32.load
      local.tee 1
      i32.const -1
      i32.add
      i32.store
      local.get 1
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 4
      i32.add
      call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h00ddb978cc2c31d9E
    end
    block  ;; label = @1
      local.get 0
      i32.load offset=36
      local.tee 1
      i32.const -2147483648
      i32.eq
      br_if 0 (;@1;)
      local.get 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.load offset=40
      local.get 1
      i32.const 1
      call $__rust_dealloc
    end
    local.get 0
    i32.load offset=56
    local.tee 1
    local.get 1
    i32.load
    local.tee 1
    i32.const -1
    i32.add
    i32.store
    block  ;; label = @1
      local.get 1
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 56
      i32.add
      call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hf1d2b099cab8a99cE
    end
    local.get 0
    i32.load offset=48
    local.tee 1
    local.get 1
    i32.load
    local.tee 1
    i32.const -1
    i32.add
    i32.store
    block  ;; label = @1
      local.get 1
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 48
      i32.add
      call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hf1d2b099cab8a99cE
    end
    local.get 0
    i32.load offset=72
    local.tee 1
    local.get 1
    i32.load
    local.tee 1
    i32.const -1
    i32.add
    i32.store
    block  ;; label = @1
      local.get 1
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 72
      i32.add
      call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h0960bc771bfafaf1E
    end
    local.get 0
    i32.const 20
    i32.add
    local.tee 2
    call $_ZN76_$LT$std..thread..spawnhook..SpawnHooks$u20$as$u20$core..ops..drop..Drop$GT$4drop17h3c4432d513a90f9bE
    block  ;; label = @1
      local.get 0
      i32.load offset=20
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.get 1
      i32.load
      local.tee 3
      i32.const -1
      i32.add
      i32.store
      local.get 3
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 2
      call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h4e00d7284bb1d392E
    end
    block  ;; label = @1
      local.get 0
      i32.load offset=16
      local.tee 4
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.load offset=12
      local.set 1
      loop  ;; label = @2
        local.get 1
        i32.load
        local.set 3
        block  ;; label = @3
          local.get 1
          i32.const 4
          i32.add
          i32.load
          local.tee 2
          i32.load
          local.tee 5
          i32.eqz
          br_if 0 (;@3;)
          local.get 3
          local.get 5
          call_indirect (type 0)
        end
        block  ;; label = @3
          local.get 2
          i32.load offset=4
          local.tee 5
          i32.eqz
          br_if 0 (;@3;)
          local.get 3
          local.get 5
          local.get 2
          i32.load offset=8
          call $__rust_dealloc
        end
        local.get 1
        i32.const 8
        i32.add
        local.set 1
        local.get 4
        i32.const -1
        i32.add
        local.tee 4
        br_if 0 (;@2;)
      end
    end
    block  ;; label = @1
      local.get 0
      i32.load offset=8
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.load offset=12
      local.get 1
      i32.const 3
      i32.shl
      i32.const 4
      call $__rust_dealloc
    end
    local.get 0
    i32.load offset=24
    local.tee 1
    local.get 1
    i32.load
    local.tee 1
    i32.const -1
    i32.add
    i32.store
    block  ;; label = @1
      local.get 1
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 24
      i32.add
      call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h5a9d89913051a5edE
    end)
  (func $_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h83c559db23fa37afE (type 0) (param i32)
    local.get 0
    call $_ZN3std6thread9spawnhook15ChildSpawnHooks3run17h36c48039c3942367E)
  (func $_ZN3std3sys9backtrace28__rust_begin_short_backtrace17he53b213f08fa5222E (type 0) (param i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 64
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 1
    i32.const 8
    i32.add
    i32.const 48
    i32.add
    local.get 0
    i32.const 48
    i32.add
    i32.load
    i32.store
    local.get 1
    i32.const 8
    i32.add
    i32.const 40
    i32.add
    local.get 0
    i32.const 40
    i32.add
    i64.load align=4
    i64.store
    local.get 1
    i32.const 8
    i32.add
    i32.const 32
    i32.add
    local.get 0
    i32.const 32
    i32.add
    i64.load align=4
    i64.store
    local.get 1
    i32.const 8
    i32.add
    i32.const 24
    i32.add
    local.get 0
    i32.const 24
    i32.add
    i64.load align=4
    i64.store
    local.get 1
    i32.const 8
    i32.add
    i32.const 16
    i32.add
    local.get 0
    i32.const 16
    i32.add
    i64.load align=4
    i64.store
    local.get 1
    i32.const 8
    i32.add
    i32.const 8
    i32.add
    local.get 0
    i32.const 8
    i32.add
    i64.load align=4
    i64.store
    local.get 1
    local.get 0
    i64.load align=4
    i64.store offset=8
    local.get 1
    i32.const 8
    i32.add
    call $_ZN10rayon_core8registry13ThreadBuilder3run17h242dc1e44c4ac4d0E
    local.get 1
    i32.const 64
    i32.add
    global.set $__stack_pointer)
  (func $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h30bcf172ba43f522E (type 4) (param i32 i32) (result i32)
    local.get 0
    i32.load
    local.get 1
    call $_ZN58_$LT$std..io..error..Error$u20$as$u20$core..fmt..Debug$GT$3fmt17h596e914530403c3cE)
  (func $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h7a03664ec6aefbbbE (type 4) (param i32 i32) (result i32)
    (local i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 0
            i32.load
            local.tee 3
            i32.load8_u
            i32.const -4
            i32.add
            i32.const 255
            i32.and
            local.tee 0
            i32.const 2
            local.get 0
            i32.const 2
            i32.lt_u
            select
            br_table 0 (;@4;) 1 (;@3;) 2 (;@2;) 0 (;@4;)
          end
          local.get 1
          i32.const 1051552
          i32.const 28
          call $_ZN4core3fmt9Formatter9write_str17ha951e874492915b9E
          local.set 0
          br 2 (;@1;)
        end
        local.get 1
        i32.const 1051580
        i32.const 26
        call $_ZN4core3fmt9Formatter9write_str17ha951e874492915b9E
        local.set 0
        br 1 (;@1;)
      end
      local.get 2
      local.get 3
      i32.store offset=12
      local.get 1
      i32.const 1051624
      i32.const 7
      local.get 2
      i32.const 12
      i32.add
      i32.const 1051608
      call $_ZN4core3fmt9Formatter25debug_tuple_field1_finish17hcdf3e8d519221f47E
      local.set 0
    end
    local.get 2
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN10rayon_core5latch9LockLatch14wait_and_reset17hd076fbaf8601b838E (type 0) (param i32)
    (local i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 0
    i32.load8_u
    local.set 2
    local.get 0
    i32.const 1
    i32.store8
    local.get 1
    local.get 2
    i32.store8 offset=7
    block  ;; label = @1
      local.get 2
      i32.const 1
      i32.eq
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 0
        i32.load8_u offset=1
        br_if 0 (;@2;)
        local.get 0
        i32.const 2
        i32.add
        local.set 2
        loop  ;; label = @3
          local.get 2
          local.get 0
          call $_ZN3std3sys4sync7condvar10no_threads7Condvar4wait17hb91e5e8042bc81e5E
          local.get 0
          i32.load8_u offset=1
          i32.const 1
          i32.ne
          br_if 0 (;@3;)
        end
      end
      local.get 0
      i32.const 0
      i32.store16 align=1
      local.get 1
      i32.const 32
      i32.add
      global.set $__stack_pointer
      return
    end
    local.get 1
    i64.const 0
    i64.store offset=20 align=4
    local.get 1
    i64.const 17179869185
    i64.store offset=12 align=4
    local.get 1
    i32.const 1050580
    i32.store offset=8
    local.get 1
    i32.const 7
    i32.add
    local.get 1
    i32.const 8
    i32.add
    call $_ZN4core9panicking13assert_failed17h9a3e3b4c3cfd5ab0E.llvm.6263538152696972293
    unreachable)
  (func $_ZN10rayon_core6unwind16resume_unwinding17h9301afba3ac4692cE (type 2) (param i32 i32)
    local.get 0
    local.get 1
    call $_ZN3std5panic13resume_unwind17h2dd9bc0fe68fc10cE
    unreachable)
  (func $_ZN10rayon_core19current_num_threads17h7db6c4811dab6784E (type 11) (result i32)
    (local i32)
    block  ;; label = @1
      block  ;; label = @2
        i32.const 0
        call $_ZN4core3ops8function6FnOnce9call_once17hf5860f3535acd078E.llvm.16897747005057573272
        i32.load
        local.tee 0
        br_if 0 (;@2;)
        call $_ZN10rayon_core8registry15global_registry17hf6a3fbcd34bc87c8E
        local.set 0
        br 1 (;@1;)
      end
      local.get 0
      i32.const 140
      i32.add
      local.set 0
    end
    local.get 0
    i32.load
    i32.load offset=260)
  (func $_ZN5alloc7raw_vec11finish_grow17h3f226ff5c3215776E.llvm.11642234386891931539 (type 12) (param i32 i32 i32 i32)
    (local i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        i32.const 0
        i32.lt_s
        br_if 0 (;@2;)
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 3
              i32.load offset=4
              i32.eqz
              br_if 0 (;@5;)
              block  ;; label = @6
                local.get 3
                i32.load offset=8
                local.tee 4
                br_if 0 (;@6;)
                block  ;; label = @7
                  local.get 2
                  br_if 0 (;@7;)
                  local.get 1
                  local.set 3
                  br 4 (;@3;)
                end
                i32.const 0
                i32.load8_u offset=1058985
                drop
                br 2 (;@4;)
              end
              local.get 3
              i32.load
              local.get 4
              local.get 1
              local.get 2
              call $__rust_realloc
              local.set 3
              br 2 (;@3;)
            end
            block  ;; label = @5
              local.get 2
              br_if 0 (;@5;)
              local.get 1
              local.set 3
              br 2 (;@3;)
            end
            i32.const 0
            i32.load8_u offset=1058985
            drop
          end
          local.get 2
          local.get 1
          call $__rust_alloc
          local.set 3
        end
        block  ;; label = @3
          local.get 3
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          local.get 2
          i32.store offset=8
          local.get 0
          local.get 3
          i32.store offset=4
          local.get 0
          i32.const 0
          i32.store
          return
        end
        local.get 0
        local.get 2
        i32.store offset=8
        local.get 0
        local.get 1
        i32.store offset=4
        br 1 (;@1;)
      end
      local.get 0
      i32.const 0
      i32.store offset=4
    end
    local.get 0
    i32.const 1
    i32.store)
  (func $_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17h7844c62910e0bc64E (type 7) (param i32 i32 i32 i32 i32)
    (local i32 i32 i32 i32 i64)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 5
    global.set $__stack_pointer
    i32.const 0
    local.set 6
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 4
          br_if 0 (;@3;)
          br 1 (;@2;)
        end
        block  ;; label = @3
          local.get 1
          local.get 2
          i32.add
          local.tee 2
          local.get 1
          i32.ge_u
          br_if 0 (;@3;)
          br 1 (;@2;)
        end
        i32.const 0
        local.set 6
        block  ;; label = @3
          local.get 3
          local.get 4
          i32.add
          i32.const -1
          i32.add
          i32.const 0
          local.get 3
          i32.sub
          i32.and
          i64.extend_i32_u
          i32.const 8
          i32.const 4
          i32.const 1
          local.get 4
          i32.const 1025
          i32.lt_u
          select
          local.get 4
          i32.const 1
          i32.eq
          select
          local.tee 7
          local.get 0
          i32.load
          local.tee 1
          i32.const 1
          i32.shl
          local.tee 8
          local.get 2
          local.get 8
          local.get 2
          i32.gt_u
          select
          local.tee 2
          local.get 7
          local.get 2
          i32.gt_u
          select
          local.tee 7
          i64.extend_i32_u
          i64.mul
          local.tee 9
          i64.const 32
          i64.shr_u
          i32.wrap_i64
          i32.eqz
          br_if 0 (;@3;)
          br 1 (;@2;)
        end
        local.get 9
        i32.wrap_i64
        local.tee 2
        i32.const -2147483648
        local.get 3
        i32.sub
        i32.gt_u
        br_if 0 (;@2;)
        block  ;; label = @3
          block  ;; label = @4
            local.get 1
            br_if 0 (;@4;)
            i32.const 0
            local.set 4
            br 1 (;@3;)
          end
          local.get 5
          local.get 1
          local.get 4
          i32.mul
          i32.store offset=28
          local.get 5
          local.get 0
          i32.load offset=4
          i32.store offset=20
          local.get 3
          local.set 4
        end
        local.get 5
        local.get 4
        i32.store offset=24
        local.get 5
        i32.const 8
        i32.add
        local.get 3
        local.get 2
        local.get 5
        i32.const 20
        i32.add
        call $_ZN5alloc7raw_vec11finish_grow17h3f226ff5c3215776E.llvm.11642234386891931539
        local.get 5
        i32.load offset=8
        i32.const 1
        i32.ne
        br_if 1 (;@1;)
        local.get 5
        i32.load offset=16
        local.set 8
        local.get 5
        i32.load offset=12
        local.set 6
      end
      local.get 6
      local.get 8
      i32.const 1051708
      call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
      unreachable
    end
    local.get 5
    i32.load offset=12
    local.set 4
    local.get 0
    local.get 7
    i32.store
    local.get 0
    local.get 4
    i32.store offset=4
    local.get 5
    i32.const 32
    i32.add
    global.set $__stack_pointer)
  (func $_ZN3std3sys4sync4once10no_threads4Once4call17hae400d22744d091dE (type 0) (param i32)
    (local i32 i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  i32.const 0
                  i32.load8_u offset=1059012
                  br_table 0 (;@7;) 2 (;@5;) 6 (;@1;) 1 (;@6;) 0 (;@7;)
                end
                i32.const 0
                i32.const 2
                i32.store8 offset=1059012
                local.get 0
                i32.load
                local.set 2
                local.get 0
                i32.const 0
                i32.store
                local.get 2
                i32.eqz
                br_if 2 (;@4;)
                local.get 2
                i32.load
                local.set 3
                i32.const 0
                i32.load8_u offset=1058985
                drop
                i32.const 1036
                i32.const 4
                call $__rust_alloc
                local.tee 2
                i32.eqz
                br_if 3 (;@3;)
                local.get 2
                i32.const 0
                i32.store offset=1032
                i32.const 0
                i32.load8_u offset=1058985
                drop
                i32.const 320
                i32.const 64
                call $__rust_alloc
                local.tee 0
                i32.eqz
                br_if 4 (;@2;)
                local.get 0
                i32.const 0
                i32.store offset=256
                local.get 0
                i32.const 0
                i32.store offset=192
                local.get 0
                local.get 2
                i32.store offset=128
                local.get 0
                local.get 2
                i32.store offset=64
                local.get 0
                i64.const 4294967297
                i64.store
                local.get 3
                local.get 0
                i32.store
                i32.const 0
                i32.const 3
                i32.store8 offset=1059012
              end
              local.get 1
              i32.const 32
              i32.add
              global.set $__stack_pointer
              return
            end
            local.get 1
            i32.const 0
            i32.store offset=24
            local.get 1
            i32.const 1
            i32.store offset=12
            local.get 1
            i32.const 1051768
            i32.store offset=8
            local.get 1
            i64.const 4
            i64.store offset=16 align=4
            local.get 1
            i32.const 8
            i32.add
            i32.const 1052592
            call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
            unreachable
          end
          i32.const 1051916
          call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
          unreachable
        end
        i32.const 4
        i32.const 1036
        call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
        unreachable
      end
      i32.const 64
      i32.const 320
      call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
      unreachable
    end
    local.get 1
    i32.const 0
    i32.store offset=24
    local.get 1
    i32.const 1
    i32.store offset=12
    local.get 1
    i32.const 1051832
    i32.store offset=8
    local.get 1
    i64.const 4
    i64.store offset=16 align=4
    local.get 1
    i32.const 8
    i32.add
    i32.const 1052592
    call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
    unreachable)
  (func $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h03850c4c46fb1eb2E (type 4) (param i32 i32) (result i32)
    (local i32)
    local.get 0
    i32.load
    local.set 0
    block  ;; label = @1
      local.get 1
      i32.load offset=28
      local.tee 2
      i32.const 16
      i32.and
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 2
        i32.const 32
        i32.and
        br_if 0 (;@2;)
        local.get 0
        local.get 1
        call $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17he1d3bba66865ae66E
        return
      end
      local.get 0
      local.get 1
      call $_ZN4core3fmt3num53_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$i32$GT$3fmt17h2e92699d27a37844E
      return
    end
    local.get 0
    local.get 1
    call $_ZN4core3fmt3num53_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$i32$GT$3fmt17hee4425a51b6b9c20E)
  (func $_ZN4core4sync6atomic23atomic_compare_exchange17hd6f3885580709ae6E.llvm.14801380999711903380 (type 8) (param i32 i32 i32 i32 i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 6
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      block  ;; label = @10
                        block  ;; label = @11
                          block  ;; label = @12
                            block  ;; label = @13
                              block  ;; label = @14
                                block  ;; label = @15
                                  block  ;; label = @16
                                    block  ;; label = @17
                                      block  ;; label = @18
                                        block  ;; label = @19
                                          block  ;; label = @20
                                            block  ;; label = @21
                                              block  ;; label = @22
                                                block  ;; label = @23
                                                  local.get 4
                                                  i32.const 255
                                                  i32.and
                                                  br_table 0 (;@23;) 1 (;@22;) 2 (;@21;) 3 (;@20;) 4 (;@19;) 0 (;@23;)
                                                end
                                                local.get 5
                                                i32.const 255
                                                i32.and
                                                br_table 4 (;@18;) 19 (;@3;) 5 (;@17;) 20 (;@2;) 6 (;@16;) 4 (;@18;)
                                              end
                                              local.get 5
                                              i32.const 255
                                              i32.and
                                              br_table 6 (;@15;) 18 (;@3;) 7 (;@14;) 19 (;@2;) 8 (;@13;) 6 (;@15;)
                                            end
                                            local.get 5
                                            i32.const 255
                                            i32.and
                                            br_table 8 (;@12;) 17 (;@3;) 9 (;@11;) 18 (;@2;) 10 (;@10;) 8 (;@12;)
                                          end
                                          local.get 5
                                          i32.const 255
                                          i32.and
                                          br_table 10 (;@9;) 16 (;@3;) 11 (;@8;) 17 (;@2;) 12 (;@7;) 10 (;@9;)
                                        end
                                        local.get 5
                                        i32.const 255
                                        i32.and
                                        br_table 12 (;@6;) 15 (;@3;) 13 (;@5;) 16 (;@2;) 14 (;@4;) 12 (;@6;)
                                      end
                                      local.get 1
                                      local.get 3
                                      local.get 1
                                      i32.load
                                      local.tee 5
                                      local.get 5
                                      local.get 2
                                      i32.eq
                                      local.tee 4
                                      select
                                      i32.store
                                      br 16 (;@1;)
                                    end
                                    local.get 1
                                    local.get 3
                                    local.get 1
                                    i32.load
                                    local.tee 5
                                    local.get 5
                                    local.get 2
                                    i32.eq
                                    local.tee 4
                                    select
                                    i32.store
                                    br 15 (;@1;)
                                  end
                                  local.get 1
                                  local.get 3
                                  local.get 1
                                  i32.load
                                  local.tee 5
                                  local.get 5
                                  local.get 2
                                  i32.eq
                                  local.tee 4
                                  select
                                  i32.store
                                  br 14 (;@1;)
                                end
                                local.get 1
                                local.get 3
                                local.get 1
                                i32.load
                                local.tee 5
                                local.get 5
                                local.get 2
                                i32.eq
                                local.tee 4
                                select
                                i32.store
                                br 13 (;@1;)
                              end
                              local.get 1
                              local.get 3
                              local.get 1
                              i32.load
                              local.tee 5
                              local.get 5
                              local.get 2
                              i32.eq
                              local.tee 4
                              select
                              i32.store
                              br 12 (;@1;)
                            end
                            local.get 1
                            local.get 3
                            local.get 1
                            i32.load
                            local.tee 5
                            local.get 5
                            local.get 2
                            i32.eq
                            local.tee 4
                            select
                            i32.store
                            br 11 (;@1;)
                          end
                          local.get 1
                          local.get 3
                          local.get 1
                          i32.load
                          local.tee 5
                          local.get 5
                          local.get 2
                          i32.eq
                          local.tee 4
                          select
                          i32.store
                          br 10 (;@1;)
                        end
                        local.get 1
                        local.get 3
                        local.get 1
                        i32.load
                        local.tee 5
                        local.get 5
                        local.get 2
                        i32.eq
                        local.tee 4
                        select
                        i32.store
                        br 9 (;@1;)
                      end
                      local.get 1
                      local.get 3
                      local.get 1
                      i32.load
                      local.tee 5
                      local.get 5
                      local.get 2
                      i32.eq
                      local.tee 4
                      select
                      i32.store
                      br 8 (;@1;)
                    end
                    local.get 1
                    local.get 3
                    local.get 1
                    i32.load
                    local.tee 5
                    local.get 5
                    local.get 2
                    i32.eq
                    local.tee 4
                    select
                    i32.store
                    br 7 (;@1;)
                  end
                  local.get 1
                  local.get 3
                  local.get 1
                  i32.load
                  local.tee 5
                  local.get 5
                  local.get 2
                  i32.eq
                  local.tee 4
                  select
                  i32.store
                  br 6 (;@1;)
                end
                local.get 1
                local.get 3
                local.get 1
                i32.load
                local.tee 5
                local.get 5
                local.get 2
                i32.eq
                local.tee 4
                select
                i32.store
                br 5 (;@1;)
              end
              local.get 1
              local.get 3
              local.get 1
              i32.load
              local.tee 5
              local.get 5
              local.get 2
              i32.eq
              local.tee 4
              select
              i32.store
              br 4 (;@1;)
            end
            local.get 1
            local.get 3
            local.get 1
            i32.load
            local.tee 5
            local.get 5
            local.get 2
            i32.eq
            local.tee 4
            select
            i32.store
            br 3 (;@1;)
          end
          local.get 1
          local.get 3
          local.get 1
          i32.load
          local.tee 5
          local.get 5
          local.get 2
          i32.eq
          local.tee 4
          select
          i32.store
          br 2 (;@1;)
        end
        local.get 6
        i32.const 0
        i32.store offset=24
        local.get 6
        i32.const 1
        i32.store offset=12
        local.get 6
        i32.const 1052064
        i32.store offset=8
        local.get 6
        i64.const 4
        i64.store offset=16 align=4
        local.get 6
        i32.const 8
        i32.add
        i32.const 1052072
        call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
        unreachable
      end
      local.get 6
      i32.const 0
      i32.store offset=24
      local.get 6
      i32.const 1
      i32.store offset=12
      local.get 6
      i32.const 1052152
      i32.store offset=8
      local.get 6
      i64.const 4
      i64.store offset=16 align=4
      local.get 6
      i32.const 8
      i32.add
      i32.const 1052160
      call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
      unreachable
    end
    local.get 0
    local.get 5
    i32.store offset=4
    local.get 0
    local.get 4
    i32.const -1
    i32.xor
    i32.const 1
    i32.and
    i32.store
    local.get 6
    i32.const 32
    i32.add
    global.set $__stack_pointer)
  (func $_ZN4core9panicking13assert_failed17hc885ec6ec2952293E.llvm.14801380999711903380 (type 12) (param i32 i32 i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 4
    global.set $__stack_pointer
    local.get 4
    local.get 1
    i32.store offset=12
    local.get 4
    local.get 0
    i32.store offset=8
    i32.const 0
    local.get 4
    i32.const 8
    i32.add
    i32.const 1052176
    local.get 4
    i32.const 12
    i32.add
    i32.const 1052176
    local.get 2
    local.get 3
    call $_ZN4core9panicking19assert_failed_inner17he4920e028524a869E
    unreachable)
  (func $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h030c17e9b75ffae6E (type 0) (param i32)
    (local i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.load
        local.tee 2
        i32.load offset=256
        i32.const -4
        i32.and
        local.tee 0
        i32.eqz
        br_if 0 (;@2;)
        loop  ;; label = @3
          local.get 1
          local.get 0
          i32.load
          local.tee 3
          i32.const 3
          i32.and
          local.tee 4
          i32.store offset=4
          local.get 4
          i32.const 1
          i32.ne
          br_if 2 (;@1;)
          local.get 0
          i32.const 1052764
          call $_ZN131_$LT$crossbeam_epoch..internal..Local$u20$as$u20$crossbeam_epoch..sync..list..IsElement$LT$crossbeam_epoch..internal..Local$GT$$GT$8finalize17hf7fb4f0f2b133fc1E
          local.get 3
          i32.const -4
          i32.and
          local.tee 0
          br_if 0 (;@3;)
        end
      end
      local.get 2
      i32.const 64
      i32.add
      call $_ZN86_$LT$crossbeam_epoch..sync..queue..Queue$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc276f115a26d1f22E
      block  ;; label = @2
        local.get 2
        i32.const -1
        i32.eq
        br_if 0 (;@2;)
        local.get 2
        local.get 2
        i32.load offset=4
        local.tee 0
        i32.const -1
        i32.add
        i32.store offset=4
        local.get 0
        i32.const 1
        i32.ne
        br_if 0 (;@2;)
        local.get 2
        i32.const 320
        i32.const 64
        call $__rust_dealloc
      end
      local.get 1
      i32.const 32
      i32.add
      global.set $__stack_pointer
      return
    end
    local.get 1
    i32.const 0
    i32.store offset=8
    local.get 1
    i32.const 4
    i32.add
    i32.const 1052348
    local.get 1
    i32.const 8
    i32.add
    i32.const 1052460
    call $_ZN4core9panicking13assert_failed17hc885ec6ec2952293E.llvm.14801380999711903380
    unreachable)
  (func $_ZN15crossbeam_epoch8deferred8Deferred3new4call17h10d9322f5bc84455E.llvm.14801380999711903380 (type 0) (param i32)
    (local i32 i32 i32 i32 i32 i32 i64 i64)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    block  ;; label = @1
      local.get 0
      i32.load
      i32.const -64
      i32.and
      local.tee 2
      i32.load offset=1032
      local.tee 3
      i32.const 65
      i32.ge_u
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 3
        i32.eqz
        br_if 0 (;@2;)
        local.get 3
        i32.const -1
        i32.add
        i32.const 268435455
        i32.and
        local.set 4
        local.get 1
        i32.const 4
        i32.add
        local.set 5
        local.get 2
        i32.const 8
        i32.or
        local.tee 6
        local.set 0
        block  ;; label = @3
          local.get 3
          i32.const 1
          i32.and
          i32.eqz
          br_if 0 (;@3;)
          local.get 1
          i32.const 8
          i32.add
          local.get 6
          i32.const 8
          i32.add
          local.tee 0
          i64.load align=4
          i64.store
          local.get 0
          i32.const 0
          i64.load offset=1052740 align=4
          i64.store align=4
          local.get 6
          i64.load align=4
          local.set 7
          local.get 6
          i32.const 0
          i64.load offset=1052732 align=4
          i64.store align=4
          local.get 1
          local.get 7
          i64.store
          local.get 5
          local.get 7
          i32.wrap_i64
          call_indirect (type 0)
          local.get 2
          i32.const 24
          i32.add
          local.set 0
        end
        local.get 4
        i32.eqz
        br_if 0 (;@2;)
        local.get 6
        local.get 3
        i32.const 4
        i32.shl
        i32.add
        local.set 4
        i32.const 0
        i64.load offset=1052732 align=4
        local.set 7
        i32.const 0
        i64.load offset=1052740 align=4
        local.set 8
        loop  ;; label = @3
          local.get 1
          i32.const 8
          i32.add
          local.tee 3
          local.get 0
          i32.const 8
          i32.add
          local.tee 6
          i64.load align=4
          i64.store
          local.get 1
          local.get 0
          i64.load align=4
          i64.store
          local.get 0
          local.get 7
          i64.store align=4
          local.get 6
          local.get 8
          i64.store align=4
          local.get 5
          local.get 1
          i32.load
          call_indirect (type 0)
          local.get 3
          local.get 0
          i32.const 24
          i32.add
          local.tee 6
          i64.load align=4
          i64.store
          local.get 1
          local.get 0
          i32.const 16
          i32.add
          local.tee 3
          i64.load align=4
          i64.store
          local.get 3
          local.get 7
          i64.store align=4
          local.get 6
          local.get 8
          i64.store align=4
          local.get 5
          local.get 1
          i32.load
          call_indirect (type 0)
          local.get 0
          i32.const 32
          i32.add
          local.tee 0
          local.get 4
          i32.ne
          br_if 0 (;@3;)
        end
      end
      local.get 2
      i32.const 1152
      i32.const 64
      call $__rust_dealloc
      local.get 1
      i32.const 16
      i32.add
      global.set $__stack_pointer
      return
    end
    local.get 3
    i32.const 64
    i32.const 1052716
    call $_ZN4core5slice5index24slice_end_index_len_fail17h07937a589bfe269aE
    unreachable)
  (func $_ZN15crossbeam_epoch8deferred8Deferred3new4call17h11a4a864381fbb19E.llvm.14801380999711903380 (type 0) (param i32)
    local.get 0
    i32.load
    i32.const -4
    i32.and
    i32.const 1036
    i32.const 4
    call $__rust_dealloc)
  (func $_ZN15crossbeam_epoch7default17default_collector17h7fa099018adad5efE (type 11) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 0
    global.set $__stack_pointer
    block  ;; label = @1
      i32.const 0
      i32.load8_u offset=1059012
      i32.const 3
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      i32.const 1059008
      i32.store offset=8
      local.get 0
      local.get 0
      i32.const 8
      i32.add
      i32.store offset=12
      local.get 0
      i32.const 12
      i32.add
      call $_ZN3std3sys4sync4once10no_threads4Once4call17hae400d22744d091dE
    end
    local.get 0
    i32.const 16
    i32.add
    global.set $__stack_pointer
    i32.const 1059008)
  (func $_ZN15crossbeam_epoch9collector9Collector8register17h6bed026672a5d855E (type 5) (param i32) (result i32)
    (local i32 i32 i32 i64 i64 i32 i32)
    global.get $__stack_pointer
    i32.const 1024
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 0
    i32.load
    local.tee 2
    local.get 2
    i32.load
    local.tee 0
    i32.const 1
    i32.add
    i32.store
    i32.const 0
    local.set 3
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.const 0
        i32.lt_s
        br_if 0 (;@2;)
        i32.const 0
        i64.load offset=1052732 align=4
        local.set 4
        i32.const 0
        i64.load offset=1052740 align=4
        local.set 5
        loop  ;; label = @3
          local.get 1
          local.get 3
          i32.add
          local.tee 0
          local.get 4
          i64.store align=4
          local.get 0
          i32.const 8
          i32.add
          local.get 5
          i64.store align=4
          local.get 0
          i32.const 16
          i32.add
          local.get 4
          i64.store align=4
          local.get 0
          i32.const 24
          i32.add
          local.get 5
          i64.store align=4
          local.get 0
          i32.const 32
          i32.add
          local.get 4
          i64.store align=4
          local.get 0
          i32.const 40
          i32.add
          local.get 5
          i64.store align=4
          local.get 0
          i32.const 48
          i32.add
          local.get 4
          i64.store align=4
          local.get 0
          i32.const 56
          i32.add
          local.get 5
          i64.store align=4
          local.get 3
          i32.const 64
          i32.add
          local.tee 3
          i32.const 1024
          i32.ne
          br_if 0 (;@3;)
        end
        i32.const 0
        i32.load8_u offset=1058985
        drop
        i32.const 1152
        i32.const 64
        call $__rust_alloc
        local.tee 6
        i32.eqz
        br_if 1 (;@1;)
        local.get 6
        local.get 2
        i32.store offset=4
        local.get 6
        i32.const 0
        i32.store
        local.get 6
        i32.const 8
        i32.add
        local.get 1
        i32.const 1024
        call $memcpy
        drop
        local.get 6
        i32.const 0
        i32.store offset=1088
        local.get 6
        i64.const 1
        i64.store offset=1040
        local.get 6
        i64.const 0
        i64.store offset=1032
        local.get 6
        local.get 2
        i32.load offset=256
        local.tee 3
        i32.store
        local.get 2
        local.get 6
        local.get 2
        i32.load offset=256
        local.tee 0
        local.get 0
        local.get 3
        i32.eq
        local.tee 3
        select
        i32.store offset=256
        block  ;; label = @3
          local.get 3
          br_if 0 (;@3;)
          loop  ;; label = @4
            local.get 6
            local.get 0
            i32.store
            local.get 2
            local.get 6
            local.get 2
            i32.load offset=256
            local.tee 3
            local.get 3
            local.get 0
            i32.eq
            select
            i32.store offset=256
            local.get 3
            local.get 0
            i32.ne
            local.set 7
            local.get 3
            local.set 0
            local.get 7
            br_if 0 (;@4;)
          end
        end
        local.get 1
        i32.const 1024
        i32.add
        global.set $__stack_pointer
        local.get 6
        return
      end
      unreachable
    end
    i32.const 64
    i32.const 1152
    call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
    unreachable)
  (func $_ZN15crossbeam_epoch8deferred8Deferred5NO_OP10no_op_call17h95b2cac4a016c451E.llvm.6279718309794500740 (type 0) (param i32))
  (func $_ZN15crossbeam_epoch5guard5Guard5flush17h32b4d49dc69f4d8aE (type 0) (param i32)
    (local i32)
    block  ;; label = @1
      local.get 0
      i32.load
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 1
        i32.load offset=1032
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.load offset=4
        i32.const 64
        i32.add
        local.get 1
        i32.const 8
        i32.add
        local.get 0
        call $_ZN15crossbeam_epoch8internal6Global8push_bag17h3fd79ea7b88a677eE.llvm.6279718309794500740
      end
      local.get 1
      i32.load offset=4
      i32.const 64
      i32.add
      local.get 0
      call $_ZN15crossbeam_epoch8internal6Global7collect17h7d10bdb777c95448E
    end)
  (func $_ZN15crossbeam_epoch8internal6Global8push_bag17h3fd79ea7b88a677eE.llvm.6279718309794500740 (type 3) (param i32 i32 i32)
    (local i32 i32 i64 i64 i32 i32)
    global.get $__stack_pointer
    i32.const 2080
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    i32.const 0
    local.set 4
    i32.const 0
    i64.load offset=1052732 align=4
    local.set 5
    i32.const 0
    i64.load offset=1052740 align=4
    local.set 6
    loop  ;; label = @1
      local.get 3
      i32.const 1056
      i32.add
      local.get 4
      i32.add
      local.tee 7
      local.get 5
      i64.store align=4
      local.get 7
      i32.const 8
      i32.add
      local.get 6
      i64.store align=4
      local.get 7
      i32.const 16
      i32.add
      local.get 5
      i64.store align=4
      local.get 7
      i32.const 24
      i32.add
      local.get 6
      i64.store align=4
      local.get 7
      i32.const 32
      i32.add
      local.get 5
      i64.store align=4
      local.get 7
      i32.const 40
      i32.add
      local.get 6
      i64.store align=4
      local.get 7
      i32.const 48
      i32.add
      local.get 5
      i64.store align=4
      local.get 7
      i32.const 56
      i32.add
      local.get 6
      i64.store align=4
      local.get 4
      i32.const 64
      i32.add
      local.tee 4
      i32.const 1024
      i32.ne
      br_if 0 (;@1;)
    end
    local.get 3
    i32.const 28
    i32.add
    local.get 1
    i32.const 1028
    call $memcpy
    drop
    local.get 1
    local.get 3
    i32.const 1056
    i32.add
    i32.const 1024
    call $memcpy
    i32.const 0
    i32.store offset=1024
    i32.const 0
    i32.load8_u offset=1058985
    drop
    local.get 0
    i32.load offset=128
    local.set 7
    block  ;; label = @1
      i32.const 1036
      i32.const 4
      call $__rust_alloc
      local.tee 4
      i32.eqz
      br_if 0 (;@1;)
      local.get 4
      local.get 3
      i32.const 28
      i32.add
      i32.const 1028
      call $memcpy
      local.tee 8
      i32.const 0
      i32.store offset=1032
      local.get 8
      local.get 7
      i32.store offset=1028
      local.get 0
      i32.const 64
      i32.add
      local.set 4
      loop  ;; label = @2
        local.get 4
        i32.load
        local.tee 1
        i32.const -4
        i32.and
        local.tee 7
        i32.const 1032
        i32.add
        local.set 0
        block  ;; label = @3
          local.get 7
          i32.load offset=1032
          local.tee 7
          i32.const 3
          i32.gt_u
          br_if 0 (;@3;)
          local.get 3
          i32.const 8
          i32.add
          local.get 0
          i32.const 0
          local.get 8
          i32.const 1
          i32.const 0
          call $_ZN4core4sync6atomic23atomic_compare_exchange17hd6f3885580709ae6E.llvm.14801380999711903380
          local.get 3
          i32.load offset=8
          i32.const 1
          i32.and
          br_if 1 (;@2;)
          local.get 3
          local.get 4
          local.get 1
          local.get 8
          i32.const 1
          i32.const 0
          call $_ZN4core4sync6atomic23atomic_compare_exchange17hd6f3885580709ae6E.llvm.14801380999711903380
          local.get 3
          i32.const 2080
          i32.add
          global.set $__stack_pointer
          return
        end
        local.get 3
        i32.const 16
        i32.add
        local.get 4
        local.get 1
        local.get 7
        i32.const 1
        i32.const 0
        call $_ZN4core4sync6atomic23atomic_compare_exchange17hd6f3885580709ae6E.llvm.14801380999711903380
        br 0 (;@2;)
      end
    end
    i32.const 4
    i32.const 1036
    call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
    unreachable)
  (func $_ZN15crossbeam_epoch8internal6Global7collect17h7d10bdb777c95448E (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i64 i64 i32 i32 i64)
    global.get $__stack_pointer
    i32.const 2128
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 0
    i32.const 64
    i32.add
    local.set 3
    local.get 0
    local.get 1
    call $_ZN15crossbeam_epoch8internal6Global11try_advance17h99b31b2a78da0fa8E
    local.set 4
    local.get 1
    i32.load
    local.tee 5
    i32.const 8
    i32.add
    local.set 6
    local.get 2
    i32.const 1080
    i32.add
    i32.const 16
    i32.add
    local.set 7
    local.get 2
    i32.const 2112
    i32.add
    i32.const 4
    i32.add
    local.set 8
    local.get 2
    i32.const 1080
    i32.add
    i32.const 4
    i32.add
    local.set 9
    i32.const 0
    local.set 10
    block  ;; label = @1
      loop  ;; label = @2
        local.get 10
        i32.const 1
        i32.add
        local.set 10
        block  ;; label = @3
          loop  ;; label = @4
            local.get 0
            i32.load
            local.tee 11
            i32.const -4
            i32.and
            local.tee 12
            i32.load offset=1032
            local.tee 1
            i32.const -4
            i32.and
            local.tee 13
            i32.eqz
            br_if 1 (;@3;)
            local.get 4
            local.get 13
            i32.const 1028
            i32.add
            i32.load
            i32.const -2
            i32.and
            i32.sub
            i32.const 4
            i32.lt_s
            br_if 1 (;@3;)
            local.get 2
            i32.const 40
            i32.add
            local.get 0
            local.get 11
            local.get 1
            i32.const 1
            i32.const 0
            call $_ZN4core4sync6atomic23atomic_compare_exchange17hd6f3885580709ae6E.llvm.14801380999711903380
            local.get 2
            i32.load offset=40
            i32.const 1
            i32.and
            br_if 0 (;@4;)
          end
          block  ;; label = @4
            local.get 11
            local.get 3
            i32.load
            i32.ne
            br_if 0 (;@4;)
            local.get 2
            i32.const 32
            i32.add
            local.get 3
            local.get 11
            local.get 1
            i32.const 1
            i32.const 0
            call $_ZN4core4sync6atomic23atomic_compare_exchange17hd6f3885580709ae6E.llvm.14801380999711903380
          end
          block  ;; label = @4
            block  ;; label = @5
              local.get 5
              br_if 0 (;@5;)
              local.get 12
              i32.const 1036
              i32.const 4
              call $__rust_dealloc
              br 1 (;@4;)
            end
            block  ;; label = @5
              local.get 5
              i32.load offset=1032
              local.tee 1
              i32.const 64
              i32.lt_u
              br_if 0 (;@5;)
              loop  ;; label = @6
                local.get 5
                i32.load offset=4
                local.set 14
                i32.const 0
                local.set 12
                loop  ;; label = @7
                  local.get 2
                  i32.const 52
                  i32.add
                  local.get 12
                  i32.add
                  local.tee 1
                  i32.const 0
                  i64.load offset=1052732 align=4
                  local.tee 15
                  i64.store align=4
                  local.get 1
                  i32.const 8
                  i32.add
                  i32.const 0
                  i64.load offset=1052740 align=4
                  local.tee 16
                  i64.store align=4
                  local.get 1
                  i32.const 16
                  i32.add
                  local.get 15
                  i64.store align=4
                  local.get 1
                  i32.const 24
                  i32.add
                  local.get 16
                  i64.store align=4
                  local.get 1
                  i32.const 32
                  i32.add
                  local.get 15
                  i64.store align=4
                  local.get 1
                  i32.const 40
                  i32.add
                  local.get 16
                  i64.store align=4
                  local.get 1
                  i32.const 48
                  i32.add
                  local.get 15
                  i64.store align=4
                  local.get 1
                  i32.const 56
                  i32.add
                  local.get 16
                  i64.store align=4
                  local.get 12
                  i32.const 64
                  i32.add
                  local.tee 12
                  i32.const 1024
                  i32.ne
                  br_if 0 (;@7;)
                end
                local.get 2
                i32.const 1080
                i32.add
                local.get 6
                i32.const 1028
                call $memcpy
                drop
                local.get 6
                local.get 2
                i32.const 52
                i32.add
                i32.const 1024
                call $memcpy
                drop
                local.get 5
                i32.const 0
                i32.store offset=1032
                i32.const 0
                i32.load8_u offset=1058985
                drop
                local.get 14
                i32.load offset=192
                local.set 1
                block  ;; label = @7
                  i32.const 1036
                  i32.const 4
                  call $__rust_alloc
                  local.tee 12
                  i32.eqz
                  br_if 0 (;@7;)
                  local.get 12
                  local.get 2
                  i32.const 1080
                  i32.add
                  i32.const 1028
                  call $memcpy
                  local.tee 17
                  i32.const 0
                  i32.store offset=1032
                  local.get 17
                  local.get 1
                  i32.store offset=1028
                  local.get 14
                  i32.const 128
                  i32.add
                  local.set 12
                  loop  ;; label = @8
                    local.get 12
                    i32.load
                    local.tee 14
                    i32.const -4
                    i32.and
                    local.tee 1
                    i32.const 1032
                    i32.add
                    local.set 18
                    block  ;; label = @9
                      local.get 1
                      i32.load offset=1032
                      local.tee 1
                      i32.const 3
                      i32.gt_u
                      br_if 0 (;@9;)
                      local.get 2
                      i32.const 16
                      i32.add
                      local.get 18
                      i32.const 0
                      local.get 17
                      i32.const 1
                      i32.const 0
                      call $_ZN4core4sync6atomic23atomic_compare_exchange17hd6f3885580709ae6E.llvm.14801380999711903380
                      local.get 2
                      i32.load offset=16
                      i32.const 1
                      i32.and
                      br_if 1 (;@8;)
                      local.get 2
                      i32.const 8
                      i32.add
                      local.get 12
                      local.get 14
                      local.get 17
                      i32.const 1
                      i32.const 0
                      call $_ZN4core4sync6atomic23atomic_compare_exchange17hd6f3885580709ae6E.llvm.14801380999711903380
                      local.get 5
                      i32.load offset=1032
                      local.tee 1
                      i32.const 63
                      i32.gt_u
                      br_if 3 (;@6;)
                      br 4 (;@5;)
                    end
                    local.get 2
                    i32.const 24
                    i32.add
                    local.get 12
                    local.get 14
                    local.get 1
                    i32.const 1
                    i32.const 0
                    call $_ZN4core4sync6atomic23atomic_compare_exchange17hd6f3885580709ae6E.llvm.14801380999711903380
                    br 0 (;@8;)
                  end
                end
              end
              i32.const 4
              i32.const 1036
              call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
              unreachable
            end
            local.get 6
            local.get 1
            i32.const 4
            i32.shl
            i32.add
            local.tee 1
            local.get 11
            i32.store offset=4
            local.get 1
            i32.const 19
            i32.store
            local.get 5
            local.get 5
            i32.load offset=1032
            i32.const 1
            i32.add
            i32.store offset=1032
          end
          local.get 13
          i32.load
          local.tee 1
          i32.eqz
          br_if 0 (;@3;)
          local.get 2
          i32.const 52
          i32.add
          local.get 13
          i32.const 4
          i32.add
          i32.const 1028
          call $memcpy
          drop
          local.get 2
          local.get 1
          i32.store offset=1080
          local.get 9
          local.get 2
          i32.const 52
          i32.add
          i32.const 1028
          call $memcpy
          drop
          local.get 2
          i32.load offset=2104
          local.tee 12
          i32.const 65
          i32.ge_u
          br_if 2 (;@1;)
          block  ;; label = @4
            local.get 12
            i32.eqz
            br_if 0 (;@4;)
            local.get 12
            i32.const -1
            i32.add
            i32.const 268435455
            i32.and
            local.set 14
            local.get 2
            i32.const 1080
            i32.add
            local.set 1
            block  ;; label = @5
              local.get 12
              i32.const 1
              i32.and
              i32.eqz
              br_if 0 (;@5;)
              local.get 2
              i32.const 1080
              i32.add
              i32.const 8
              i32.add
              local.tee 1
              i64.load
              local.set 15
              local.get 1
              i32.const 0
              i64.load offset=1052740 align=4
              i64.store
              local.get 2
              i32.const 2112
              i32.add
              i32.const 8
              i32.add
              local.get 15
              i64.store
              local.get 2
              local.get 2
              i64.load offset=1080
              local.tee 15
              i64.store offset=2112
              local.get 2
              i32.const 0
              i64.load offset=1052732 align=4
              i64.store offset=1080
              local.get 8
              local.get 15
              i32.wrap_i64
              call_indirect (type 0)
              local.get 7
              local.set 1
            end
            local.get 14
            i32.eqz
            br_if 0 (;@4;)
            local.get 2
            i32.const 1080
            i32.add
            local.get 12
            i32.const 4
            i32.shl
            i32.add
            local.set 18
            loop  ;; label = @5
              local.get 1
              i64.load align=4
              local.set 15
              local.get 1
              i32.const 0
              i64.load offset=1052732 align=4
              local.tee 16
              i64.store align=4
              local.get 2
              i32.const 2112
              i32.add
              i32.const 8
              i32.add
              local.tee 12
              local.get 1
              i32.const 8
              i32.add
              local.tee 14
              i64.load align=4
              i64.store
              local.get 14
              i32.const 0
              i64.load offset=1052740 align=4
              local.tee 19
              i64.store align=4
              local.get 2
              local.get 15
              i64.store offset=2112
              local.get 8
              local.get 15
              i32.wrap_i64
              call_indirect (type 0)
              local.get 12
              local.get 1
              i32.const 24
              i32.add
              local.tee 14
              i64.load align=4
              i64.store
              local.get 1
              i32.const 16
              i32.add
              local.tee 12
              i64.load align=4
              local.set 15
              local.get 12
              local.get 16
              i64.store align=4
              local.get 14
              local.get 19
              i64.store align=4
              local.get 2
              local.get 15
              i64.store offset=2112
              local.get 8
              local.get 15
              i32.wrap_i64
              call_indirect (type 0)
              local.get 1
              i32.const 32
              i32.add
              local.tee 1
              local.get 18
              i32.ne
              br_if 0 (;@5;)
            end
          end
          local.get 10
          i32.const 8
          i32.ne
          br_if 1 (;@2;)
        end
      end
      local.get 2
      i32.const 2128
      i32.add
      global.set $__stack_pointer
      return
    end
    local.get 12
    i32.const 64
    i32.const 1052716
    call $_ZN4core5slice5index24slice_end_index_len_fail17h07937a589bfe269aE
    unreachable)
  (func $_ZN15crossbeam_epoch8internal5Local8finalize17ha822a731c89268d9E (type 0) (param i32)
    (local i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 0
    i32.const 1
    i32.store offset=1040
    local.get 1
    local.get 0
    i32.store offset=12
    block  ;; label = @1
      local.get 0
      i32.load offset=1036
      local.tee 2
      i32.const -1
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      local.get 2
      i32.const 1
      i32.add
      i32.store offset=1036
      block  ;; label = @2
        local.get 2
        br_if 0 (;@2;)
        local.get 0
        i32.load offset=4
        i32.load offset=192
        local.set 2
        local.get 0
        local.get 0
        i32.load offset=1044
        local.tee 3
        i32.const 1
        i32.add
        i32.store offset=1044
        local.get 0
        local.get 2
        i32.const 1
        i32.or
        i32.store offset=1088
        local.get 3
        i32.const 127
        i32.and
        br_if 0 (;@2;)
        local.get 0
        i32.load offset=4
        i32.const 64
        i32.add
        local.get 1
        i32.const 12
        i32.add
        call $_ZN15crossbeam_epoch8internal6Global7collect17h7d10bdb777c95448E
      end
      local.get 1
      i32.load offset=12
      local.set 2
      local.get 0
      i32.load offset=4
      i32.const 64
      i32.add
      local.get 0
      i32.const 8
      i32.add
      local.get 0
      call $_ZN15crossbeam_epoch8internal6Global8push_bag17h3fd79ea7b88a677eE.llvm.6279718309794500740
      block  ;; label = @2
        local.get 2
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 2
        i32.load offset=1036
        local.tee 3
        i32.const -1
        i32.add
        i32.store offset=1036
        local.get 3
        i32.const 1
        i32.ne
        br_if 0 (;@2;)
        local.get 2
        i32.const 0
        i32.store offset=1088
        local.get 2
        i32.load offset=1040
        br_if 0 (;@2;)
        local.get 2
        call $_ZN15crossbeam_epoch8internal5Local8finalize17ha822a731c89268d9E
      end
      local.get 0
      i32.const 0
      i32.store offset=1040
      local.get 0
      local.get 0
      i32.load
      i32.const 1
      i32.or
      i32.store
      local.get 1
      local.get 0
      i32.load offset=4
      local.tee 0
      i32.store offset=8
      local.get 0
      local.get 0
      i32.load
      local.tee 2
      i32.const -1
      i32.add
      i32.store
      block  ;; label = @2
        local.get 2
        i32.const 1
        i32.ne
        br_if 0 (;@2;)
        local.get 1
        i32.const 8
        i32.add
        call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17h030c17e9b75ffae6E
      end
      local.get 1
      i32.const 16
      i32.add
      global.set $__stack_pointer
      return
    end
    i32.const 1052748
    call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
    unreachable)
  (func $_ZN15crossbeam_epoch8internal6Global11try_advance17h99b31b2a78da0fa8E (type 4) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32)
    local.get 0
    i32.const 192
    i32.add
    local.set 2
    local.get 0
    i32.load offset=192
    local.set 3
    local.get 0
    i32.load offset=128
    local.set 4
    block  ;; label = @1
      loop  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 3
            i32.const -4
            i32.and
            local.tee 5
            i32.eqz
            br_if 0 (;@4;)
            local.get 3
            local.set 6
            loop  ;; label = @5
              local.get 5
              i32.load
              local.tee 3
              i32.const 3
              i32.and
              i32.const 1
              i32.ne
              br_if 2 (;@3;)
              local.get 2
              local.get 3
              i32.const -4
              i32.and
              local.tee 5
              local.get 2
              i32.load
              local.tee 3
              local.get 3
              local.get 6
              i32.eq
              select
              i32.store
              block  ;; label = @6
                block  ;; label = @7
                  local.get 3
                  local.get 6
                  i32.ne
                  br_if 0 (;@7;)
                  local.get 6
                  i32.const -4
                  i32.and
                  local.get 1
                  call $_ZN131_$LT$crossbeam_epoch..internal..Local$u20$as$u20$crossbeam_epoch..sync..list..IsElement$LT$crossbeam_epoch..internal..Local$GT$$GT$8finalize17hf7fb4f0f2b133fc1E
                  local.get 5
                  local.set 6
                  br 1 (;@6;)
                end
                local.get 3
                local.set 6
              end
              local.get 6
              i32.const 3
              i32.and
              br_if 4 (;@1;)
              local.get 6
              local.set 5
              local.get 6
              br_if 0 (;@5;)
            end
          end
          local.get 0
          local.get 4
          i32.const 2
          i32.add
          local.tee 4
          i32.store offset=128
          br 2 (;@1;)
        end
        local.get 5
        local.set 2
        local.get 5
        i32.load offset=1088
        local.tee 6
        i32.const 1
        i32.and
        i32.eqz
        br_if 0 (;@2;)
        local.get 5
        local.set 2
        local.get 6
        i32.const -2
        i32.and
        local.get 4
        i32.eq
        br_if 0 (;@2;)
      end
    end
    local.get 4)
  (func $_ZN131_$LT$crossbeam_epoch..internal..Local$u20$as$u20$crossbeam_epoch..sync..list..IsElement$LT$crossbeam_epoch..internal..Local$GT$$GT$8finalize17hf7fb4f0f2b133fc1E (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i64 i64 i64)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    local.get 0
    i32.const 60
    i32.and
    local.tee 3
    i32.store offset=4
    block  ;; label = @1
      block  ;; label = @2
        local.get 3
        br_if 0 (;@2;)
        block  ;; label = @3
          block  ;; label = @4
            local.get 1
            i32.load
            local.tee 3
            br_if 0 (;@4;)
            local.get 0
            i32.load offset=1032
            local.tee 4
            i32.const 65
            i32.ge_u
            br_if 3 (;@1;)
            block  ;; label = @5
              local.get 4
              i32.eqz
              br_if 0 (;@5;)
              local.get 4
              i32.const -1
              i32.add
              i32.const 268435455
              i32.and
              local.set 5
              local.get 2
              i32.const 8
              i32.add
              i32.const 4
              i32.add
              local.set 1
              local.get 0
              i32.const 8
              i32.add
              local.tee 6
              local.set 3
              block  ;; label = @6
                local.get 4
                i32.const 1
                i32.and
                i32.eqz
                br_if 0 (;@6;)
                local.get 6
                i64.load align=4
                local.set 7
                local.get 6
                i32.const 0
                i64.load offset=1052732 align=4
                i64.store align=4
                local.get 2
                i32.const 8
                i32.add
                i32.const 8
                i32.add
                local.get 6
                i32.const 8
                i32.add
                local.tee 3
                i64.load align=4
                i64.store
                local.get 3
                i32.const 0
                i64.load offset=1052740 align=4
                i64.store align=4
                local.get 2
                local.get 7
                i64.store offset=8
                local.get 1
                local.get 7
                i32.wrap_i64
                call_indirect (type 0)
                local.get 0
                i32.const 24
                i32.add
                local.set 3
              end
              local.get 5
              i32.eqz
              br_if 0 (;@5;)
              local.get 6
              local.get 4
              i32.const 4
              i32.shl
              i32.add
              local.set 5
              i32.const 0
              i64.load offset=1052732 align=4
              local.set 8
              i32.const 0
              i64.load offset=1052740 align=4
              local.set 9
              loop  ;; label = @6
                local.get 3
                i64.load align=4
                local.set 7
                local.get 3
                local.get 8
                i64.store align=4
                local.get 2
                i32.const 8
                i32.add
                i32.const 8
                i32.add
                local.tee 4
                local.get 3
                i32.const 8
                i32.add
                local.tee 6
                i64.load align=4
                i64.store
                local.get 6
                local.get 9
                i64.store align=4
                local.get 2
                local.get 7
                i64.store offset=8
                local.get 1
                local.get 7
                i32.wrap_i64
                call_indirect (type 0)
                local.get 4
                local.get 3
                i32.const 24
                i32.add
                local.tee 6
                i64.load align=4
                i64.store
                local.get 3
                i32.const 16
                i32.add
                local.tee 4
                i64.load align=4
                local.set 7
                local.get 4
                local.get 8
                i64.store align=4
                local.get 6
                local.get 9
                i64.store align=4
                local.get 2
                local.get 7
                i64.store offset=8
                local.get 1
                local.get 7
                i32.wrap_i64
                call_indirect (type 0)
                local.get 3
                i32.const 32
                i32.add
                local.tee 3
                local.get 5
                i32.ne
                br_if 0 (;@6;)
              end
            end
            local.get 0
            i32.const 1152
            i32.const 64
            call $__rust_dealloc
            br 1 (;@3;)
          end
          local.get 3
          i32.const 8
          i32.add
          local.set 1
          block  ;; label = @4
            local.get 3
            i32.load offset=1032
            local.tee 4
            i32.const 64
            i32.lt_u
            br_if 0 (;@4;)
            loop  ;; label = @5
              local.get 3
              i32.load offset=4
              i32.const 64
              i32.add
              local.get 1
              local.get 3
              call $_ZN15crossbeam_epoch8internal6Global8push_bag17h3fd79ea7b88a677eE.llvm.6279718309794500740
              local.get 3
              i32.load offset=1032
              local.tee 4
              i32.const 63
              i32.gt_u
              br_if 0 (;@5;)
            end
          end
          local.get 1
          local.get 4
          i32.const 4
          i32.shl
          i32.add
          local.tee 1
          local.get 0
          i32.store offset=4
          local.get 1
          i32.const 20
          i32.store
          local.get 3
          local.get 3
          i32.load offset=1032
          i32.const 1
          i32.add
          i32.store offset=1032
        end
        local.get 2
        i32.const 32
        i32.add
        global.set $__stack_pointer
        return
      end
      local.get 2
      i64.const 0
      i64.store offset=20 align=4
      local.get 2
      i64.const 17179869185
      i64.store offset=12 align=4
      local.get 2
      i32.const 1052212
      i32.store offset=8
      local.get 2
      i32.const 4
      i32.add
      i32.const 1052220
      local.get 2
      i32.const 8
      i32.add
      i32.const 1052332
      call $_ZN4core9panicking13assert_failed17hc885ec6ec2952293E.llvm.14801380999711903380
      unreachable
    end
    local.get 4
    i32.const 64
    i32.const 1052716
    call $_ZN4core5slice5index24slice_end_index_len_fail17h07937a589bfe269aE
    unreachable)
  (func $_ZN15crossbeam_epoch8internal5Local5defer17h86d466552d8e8645E (type 3) (param i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i64)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 1
    i32.load
    local.set 4
    local.get 3
    i32.const 16
    i32.add
    i32.const 8
    i32.add
    local.tee 5
    local.get 1
    i32.const 12
    i32.add
    i32.load
    i32.store
    local.get 3
    local.get 1
    i64.load offset=4 align=4
    i64.store offset=16
    local.get 0
    i32.const 8
    i32.add
    local.set 6
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.load offset=1032
        local.tee 7
        i32.const 64
        i32.lt_u
        br_if 0 (;@2;)
        local.get 4
        i32.eqz
        br_if 1 (;@1;)
        local.get 1
        i32.const 4
        i32.add
        local.set 8
        loop  ;; label = @3
          local.get 3
          i32.const 8
          i32.add
          local.tee 7
          local.get 3
          i32.const 16
          i32.add
          i32.const 8
          i32.add
          local.tee 9
          i32.load
          i32.store
          local.get 3
          local.get 3
          i64.load offset=16
          i64.store
          local.get 0
          i32.load offset=4
          i32.const 64
          i32.add
          local.get 6
          local.get 3
          call $_ZN15crossbeam_epoch8internal6Global8push_bag17h3fd79ea7b88a677eE.llvm.6279718309794500740
          local.get 1
          local.get 4
          i32.store
          local.get 8
          local.get 3
          i64.load
          local.tee 10
          i64.store align=4
          local.get 8
          i32.const 8
          i32.add
          local.get 7
          i32.load
          local.tee 7
          i32.store
          local.get 9
          local.get 7
          i32.store
          local.get 3
          local.get 10
          i64.store offset=16
          local.get 0
          i32.load offset=1032
          local.tee 7
          i32.const 64
          i32.ge_u
          br_if 0 (;@3;)
        end
      end
      local.get 6
      local.get 7
      i32.const 4
      i32.shl
      i32.add
      local.tee 8
      local.get 4
      i32.store
      local.get 8
      local.get 3
      i64.load offset=16
      i64.store offset=4 align=4
      local.get 0
      local.get 0
      i32.load offset=1032
      i32.const 1
      i32.add
      i32.store offset=1032
      local.get 8
      i32.const 12
      i32.add
      local.get 5
      i32.load
      i32.store
    end
    local.get 3
    i32.const 32
    i32.add
    global.set $__stack_pointer)
  (func $_ZN86_$LT$crossbeam_epoch..sync..queue..Queue$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc276f115a26d1f22E (type 0) (param i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i64 i64 i64)
    global.get $__stack_pointer
    i32.const 1072
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 0
    i32.const 64
    i32.add
    local.set 2
    local.get 1
    i32.const 24
    i32.add
    i32.const 16
    i32.add
    local.set 3
    local.get 1
    i32.const 1056
    i32.add
    i32.const 4
    i32.add
    local.set 4
    local.get 1
    i32.const 24
    i32.add
    i32.const 4
    i32.add
    local.set 5
    block  ;; label = @1
      block  ;; label = @2
        loop  ;; label = @3
          local.get 0
          i32.load
          local.tee 6
          i32.const -4
          i32.and
          local.tee 7
          i32.load offset=1032
          local.tee 8
          i32.const -4
          i32.and
          local.tee 9
          i32.eqz
          br_if 1 (;@2;)
          local.get 1
          i32.const 16
          i32.add
          local.get 0
          local.get 6
          local.get 8
          i32.const 1
          i32.const 0
          call $_ZN4core4sync6atomic23atomic_compare_exchange17hd6f3885580709ae6E.llvm.14801380999711903380
          local.get 1
          i32.load offset=16
          i32.const 1
          i32.and
          br_if 0 (;@3;)
          block  ;; label = @4
            local.get 6
            local.get 2
            i32.load
            i32.ne
            br_if 0 (;@4;)
            local.get 1
            i32.const 8
            i32.add
            local.get 2
            local.get 6
            local.get 8
            i32.const 1
            i32.const 0
            call $_ZN4core4sync6atomic23atomic_compare_exchange17hd6f3885580709ae6E.llvm.14801380999711903380
          end
          local.get 7
          i32.const 1036
          i32.const 4
          call $__rust_dealloc
          local.get 1
          local.get 9
          i32.load
          local.tee 6
          i32.store offset=24
          local.get 5
          local.get 9
          i32.const 4
          i32.add
          i32.const 1028
          call $memmove
          drop
          local.get 6
          i32.eqz
          br_if 1 (;@2;)
          local.get 1
          i32.load offset=1048
          local.tee 8
          i32.const 65
          i32.ge_u
          br_if 2 (;@1;)
          local.get 8
          i32.eqz
          br_if 0 (;@3;)
          local.get 8
          i32.const -1
          i32.add
          i32.const 268435455
          i32.and
          local.set 9
          local.get 1
          i32.const 24
          i32.add
          local.set 6
          block  ;; label = @4
            local.get 8
            i32.const 1
            i32.and
            i32.eqz
            br_if 0 (;@4;)
            local.get 1
            i32.const 24
            i32.add
            i32.const 8
            i32.add
            local.tee 6
            i64.load
            local.set 10
            local.get 6
            i32.const 0
            i64.load offset=1052740 align=4
            i64.store
            local.get 1
            i32.const 1056
            i32.add
            i32.const 8
            i32.add
            local.get 10
            i64.store
            local.get 1
            local.get 1
            i64.load offset=24
            local.tee 10
            i64.store offset=1056
            local.get 1
            i32.const 0
            i64.load offset=1052732 align=4
            i64.store offset=24
            local.get 4
            local.get 10
            i32.wrap_i64
            call_indirect (type 0)
            local.get 3
            local.set 6
          end
          local.get 9
          i32.eqz
          br_if 0 (;@3;)
          local.get 1
          i32.const 24
          i32.add
          local.get 8
          i32.const 4
          i32.shl
          i32.add
          local.set 7
          loop  ;; label = @4
            local.get 6
            i64.load align=4
            local.set 10
            local.get 6
            i32.const 0
            i64.load offset=1052732 align=4
            local.tee 11
            i64.store align=4
            local.get 1
            i32.const 1056
            i32.add
            i32.const 8
            i32.add
            local.tee 8
            local.get 6
            i32.const 8
            i32.add
            local.tee 9
            i64.load align=4
            i64.store
            local.get 9
            i32.const 0
            i64.load offset=1052740 align=4
            local.tee 12
            i64.store align=4
            local.get 1
            local.get 10
            i64.store offset=1056
            local.get 4
            local.get 10
            i32.wrap_i64
            call_indirect (type 0)
            local.get 8
            local.get 6
            i32.const 24
            i32.add
            local.tee 9
            i64.load align=4
            i64.store
            local.get 6
            i32.const 16
            i32.add
            local.tee 8
            i64.load align=4
            local.set 10
            local.get 8
            local.get 11
            i64.store align=4
            local.get 9
            local.get 12
            i64.store align=4
            local.get 1
            local.get 10
            i64.store offset=1056
            local.get 4
            local.get 10
            i32.wrap_i64
            call_indirect (type 0)
            local.get 6
            i32.const 32
            i32.add
            local.tee 6
            local.get 7
            i32.ne
            br_if 0 (;@4;)
            br 1 (;@3;)
          end
        end
      end
      local.get 0
      i32.load
      i32.const -4
      i32.and
      i32.const 1036
      i32.const 4
      call $__rust_dealloc
      local.get 1
      i32.const 1072
      i32.add
      global.set $__stack_pointer
      return
    end
    local.get 8
    i32.const 64
    i32.const 1052716
    call $_ZN4core5slice5index24slice_end_index_len_fail17h07937a589bfe269aE
    unreachable)
  (func $_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17h0ef14a6024109e55E (type 2) (param i32 i32)
    local.get 0
    i64.const 412250589670679012
    i64.store offset=8
    local.get 0
    i64.const -4225691107682626055
    i64.store)
  (func $_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17h419978c7402a9818E (type 2) (param i32 i32)
    local.get 0
    i64.const -6963440977467231692
    i64.store offset=8
    local.get 0
    i64.const -2264427854996153059
    i64.store)
  (func $_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17h4c46e3b945c67ec3E (type 2) (param i32 i32)
    local.get 0
    i64.const 7199936582794304877
    i64.store offset=8
    local.get 0
    i64.const -5076933981314334344
    i64.store)
  (func $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h16d7108310351cefE (type 4) (param i32 i32) (result i32)
    (local i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 0
    i32.load
    local.tee 0
    i32.load offset=8
    local.set 3
    local.get 0
    i32.load offset=4
    local.set 0
    local.get 2
    i32.const 4
    i32.add
    local.get 1
    call $_ZN4core3fmt9Formatter10debug_list17h4a77cc254546a1a7E
    block  ;; label = @1
      local.get 3
      i32.eqz
      br_if 0 (;@1;)
      loop  ;; label = @2
        local.get 2
        local.get 0
        i32.store offset=12
        local.get 2
        i32.const 4
        i32.add
        local.get 2
        i32.const 12
        i32.add
        i32.const 1052768
        call $_ZN4core3fmt8builders8DebugSet5entry17h4dbe0c39ee61b1a5E
        drop
        local.get 0
        i32.const 1
        i32.add
        local.set 0
        local.get 3
        i32.const -1
        i32.add
        local.tee 3
        br_if 0 (;@2;)
      end
    end
    local.get 2
    i32.const 4
    i32.add
    call $_ZN4core3fmt8builders9DebugList6finish17h522bde96a5fb0888E
    local.set 0
    local.get 2
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h76b2378a35984c0bE (type 4) (param i32 i32) (result i32)
    (local i32)
    local.get 0
    i32.load
    local.set 0
    block  ;; label = @1
      local.get 1
      i32.load offset=28
      local.tee 2
      i32.const 16
      i32.and
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 2
        i32.const 32
        i32.and
        br_if 0 (;@2;)
        local.get 0
        local.get 1
        call $_ZN4core3fmt3num3imp51_$LT$impl$u20$core..fmt..Display$u20$for$u20$u8$GT$3fmt17he72bf047e6e0be63E
        return
      end
      local.get 0
      local.get 1
      call $_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$i8$GT$3fmt17h7609c3b52c3b7d69E
      return
    end
    local.get 0
    local.get 1
    call $_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$i8$GT$3fmt17hdee09822fcf3ff73E)
  (func $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hcf27825cef0723b2E (type 4) (param i32 i32) (result i32)
    local.get 0
    i32.load
    local.tee 0
    i32.load
    local.get 1
    local.get 0
    i32.const 4
    i32.add
    i32.load
    i32.load offset=12
    call_indirect (type 4))
  (func $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hd222f1b27591d1e4E (type 4) (param i32 i32) (result i32)
    local.get 0
    i32.load
    local.get 0
    i32.load offset=4
    local.get 1
    call $_ZN40_$LT$str$u20$as$u20$core..fmt..Debug$GT$3fmt17hd254edd9c8a21e04E)
  (func $_ZN4core3fmt3num50_$LT$impl$u20$core..fmt..Debug$u20$for$u20$i32$GT$3fmt17h5ae1eb5c912fa259E (type 4) (param i32 i32) (result i32)
    (local i32)
    block  ;; label = @1
      local.get 1
      i32.load offset=28
      local.tee 2
      i32.const 16
      i32.and
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 2
        i32.const 32
        i32.and
        br_if 0 (;@2;)
        local.get 0
        local.get 1
        call $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i32$GT$3fmt17hfd770d1228523106E
        return
      end
      local.get 0
      local.get 1
      call $_ZN4core3fmt3num53_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$i32$GT$3fmt17h2e92699d27a37844E
      return
    end
    local.get 0
    local.get 1
    call $_ZN4core3fmt3num53_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$i32$GT$3fmt17hee4425a51b6b9c20E)
  (func $_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..Debug$u20$for$u20$usize$GT$3fmt17h8418712dcb2dbdd0E (type 4) (param i32 i32) (result i32)
    (local i32)
    block  ;; label = @1
      local.get 1
      i32.load offset=28
      local.tee 2
      i32.const 16
      i32.and
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 2
        i32.const 32
        i32.and
        br_if 0 (;@2;)
        local.get 0
        local.get 1
        call $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17he1d3bba66865ae66E
        return
      end
      local.get 0
      local.get 1
      call $_ZN4core3fmt3num53_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$i32$GT$3fmt17h2e92699d27a37844E
      return
    end
    local.get 0
    local.get 1
    call $_ZN4core3fmt3num53_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$i32$GT$3fmt17hee4425a51b6b9c20E)
  (func $_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17h15fb7f1ec1c750fcE (type 7) (param i32 i32 i32 i32 i32)
    (local i32 i32 i32 i32 i64)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 5
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 1
          local.get 2
          i32.add
          local.tee 2
          local.get 1
          i32.ge_u
          br_if 0 (;@3;)
          i32.const 0
          local.set 6
          br 1 (;@2;)
        end
        i32.const 0
        local.set 6
        block  ;; label = @3
          local.get 3
          local.get 4
          i32.add
          i32.const -1
          i32.add
          i32.const 0
          local.get 3
          i32.sub
          i32.and
          i64.extend_i32_u
          i32.const 8
          i32.const 4
          local.get 4
          i32.const 1
          i32.eq
          select
          local.tee 7
          local.get 0
          i32.load
          local.tee 1
          i32.const 1
          i32.shl
          local.tee 8
          local.get 2
          local.get 8
          local.get 2
          i32.gt_u
          select
          local.tee 2
          local.get 7
          local.get 2
          i32.gt_u
          select
          local.tee 7
          i64.extend_i32_u
          i64.mul
          local.tee 9
          i64.const 32
          i64.shr_u
          i32.wrap_i64
          i32.eqz
          br_if 0 (;@3;)
          br 1 (;@2;)
        end
        local.get 9
        i32.wrap_i64
        local.tee 2
        i32.const -2147483648
        local.get 3
        i32.sub
        i32.gt_u
        br_if 0 (;@2;)
        block  ;; label = @3
          block  ;; label = @4
            local.get 1
            br_if 0 (;@4;)
            i32.const 0
            local.set 4
            br 1 (;@3;)
          end
          local.get 5
          local.get 1
          local.get 4
          i32.mul
          i32.store offset=28
          local.get 5
          local.get 0
          i32.load offset=4
          i32.store offset=20
          local.get 3
          local.set 4
        end
        local.get 5
        local.get 4
        i32.store offset=24
        local.get 5
        i32.const 8
        i32.add
        local.get 3
        local.get 2
        local.get 5
        i32.const 20
        i32.add
        call $_ZN5alloc7raw_vec11finish_grow17h7ff1f438bc526413E
        local.get 5
        i32.load offset=8
        i32.const 1
        i32.ne
        br_if 1 (;@1;)
        local.get 5
        i32.load offset=16
        local.set 8
        local.get 5
        i32.load offset=12
        local.set 6
      end
      local.get 6
      local.get 8
      i32.const 1053044
      call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
      unreachable
    end
    local.get 5
    i32.load offset=12
    local.set 3
    local.get 0
    local.get 7
    i32.store
    local.get 0
    local.get 3
    i32.store offset=4
    local.get 5
    i32.const 32
    i32.add
    global.set $__stack_pointer)
  (func $_ZN4core3fmt5Write9write_fmt17h8c80195cd11d832dE (type 4) (param i32 i32) (result i32)
    local.get 0
    i32.const 1053100
    local.get 1
    call $_ZN4core3fmt5write17hcf5d300c090957a7E)
  (func $_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17hea3cf74d7f6052eaE (type 0) (param i32)
    (local i32)
    block  ;; label = @1
      local.get 0
      i32.load
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.load offset=4
      local.get 1
      i32.const 1
      call $__rust_dealloc
    end)
  (func $_ZN4core3ptr48drop_in_place$LT$alloc..ffi..c_str..NulError$GT$17h4cf020dd6404566eE (type 0) (param i32)
    (local i32)
    block  ;; label = @1
      local.get 0
      i32.load
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.load offset=4
      local.get 1
      i32.const 1
      call $__rust_dealloc
    end)
  (func $_ZN4core3ptr55drop_in_place$LT$std..thread..spawnhook..SpawnHooks$GT$17hdd938607e6022e9bE (type 0) (param i32)
    (local i32 i32 i32 i32 i32)
    local.get 0
    i32.load
    local.set 1
    local.get 0
    i32.const 0
    i32.store
    block  ;; label = @1
      local.get 1
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        loop  ;; label = @3
          local.get 1
          local.get 1
          i32.load
          local.tee 2
          i32.const -1
          i32.add
          i32.store
          local.get 2
          i32.const 1
          i32.ne
          br_if 1 (;@2;)
          local.get 1
          i32.load offset=16
          local.set 3
          local.get 1
          i32.load offset=12
          local.set 2
          local.get 1
          i32.load offset=8
          local.set 4
          block  ;; label = @4
            local.get 1
            i32.const -1
            i32.eq
            br_if 0 (;@4;)
            local.get 1
            local.get 1
            i32.load offset=4
            local.tee 5
            i32.const -1
            i32.add
            i32.store offset=4
            local.get 5
            i32.const 1
            i32.ne
            br_if 0 (;@4;)
            local.get 1
            i32.const 20
            i32.const 4
            call $__rust_dealloc
          end
          local.get 4
          i32.eqz
          br_if 1 (;@2;)
          block  ;; label = @4
            local.get 2
            i32.load
            local.tee 1
            i32.eqz
            br_if 0 (;@4;)
            local.get 4
            local.get 1
            call_indirect (type 0)
          end
          block  ;; label = @4
            local.get 2
            i32.load offset=4
            local.tee 1
            i32.eqz
            br_if 0 (;@4;)
            local.get 4
            local.get 1
            local.get 2
            i32.load offset=8
            call $__rust_dealloc
          end
          local.get 3
          local.set 1
          local.get 3
          br_if 0 (;@3;)
        end
      end
      local.get 0
      i32.load
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.get 1
      i32.load
      local.tee 2
      i32.const -1
      i32.add
      i32.store
      local.get 2
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hc35f663fd93f47d3E
    end)
  (func $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hc35f663fd93f47d3E (type 0) (param i32)
    (local i32 i32 i32)
    local.get 0
    i32.load
    local.tee 0
    i32.load offset=8
    local.set 1
    block  ;; label = @1
      local.get 0
      i32.const 12
      i32.add
      i32.load
      local.tee 2
      i32.load
      local.tee 3
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.get 3
      call_indirect (type 0)
    end
    block  ;; label = @1
      local.get 2
      i32.load offset=4
      local.tee 3
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.get 3
      local.get 2
      i32.load offset=8
      call $__rust_dealloc
    end
    block  ;; label = @1
      local.get 0
      i32.load offset=16
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 2
      i32.load
      local.tee 1
      i32.const -1
      i32.add
      i32.store
      local.get 1
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 16
      i32.add
      call $_ZN5alloc4sync16Arc$LT$T$C$A$GT$9drop_slow17hc35f663fd93f47d3E
    end
    block  ;; label = @1
      local.get 0
      i32.const -1
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      local.get 0
      i32.load offset=4
      local.tee 2
      i32.const -1
      i32.add
      i32.store offset=4
      local.get 2
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.const 20
      i32.const 4
      call $__rust_dealloc
    end)
  (func $_ZN4core3ptr71drop_in_place$LT$std..panicking..rust_panic_without_hook..RewrapBox$GT$17hf26041a55470cdfdE (type 0) (param i32)
    (local i32 i32)
    local.get 0
    i32.load
    local.set 1
    block  ;; label = @1
      local.get 0
      i32.load offset=4
      local.tee 0
      i32.load
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.get 2
      call_indirect (type 0)
    end
    block  ;; label = @1
      local.get 0
      i32.load offset=4
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.get 2
      local.get 0
      i32.load offset=8
      call $__rust_dealloc
    end)
  (func $_ZN4core3ptr77drop_in_place$LT$std..panicking..begin_panic_handler..FormatStringPayload$GT$17h3ce79a741b975d36E (type 0) (param i32)
    (local i32)
    block  ;; label = @1
      local.get 0
      i32.load
      local.tee 1
      i32.const -2147483648
      i32.or
      i32.const -2147483648
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      i32.load offset=4
      local.get 1
      i32.const 1
      call $__rust_dealloc
    end)
  (func $_ZN4core5panic12PanicPayload6as_str17h7ffa20843f2d9518E (type 2) (param i32 i32)
    local.get 0
    i32.const 0
    i32.store)
  (func $_ZN58_$LT$alloc..string..String$u20$as$u20$core..fmt..Debug$GT$3fmt17h3e6b0c9880e0a14aE (type 4) (param i32 i32) (result i32)
    local.get 0
    i32.load offset=4
    local.get 0
    i32.load offset=8
    local.get 1
    call $_ZN40_$LT$str$u20$as$u20$core..fmt..Debug$GT$3fmt17hd254edd9c8a21e04E)
  (func $_ZN58_$LT$alloc..string..String$u20$as$u20$core..fmt..Write$GT$10write_char17h01d044b6dc206b9eE (type 4) (param i32 i32) (result i32)
    (local i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.const 128
        i32.lt_u
        br_if 0 (;@2;)
        local.get 2
        i32.const 0
        i32.store offset=12
        block  ;; label = @3
          block  ;; label = @4
            local.get 1
            i32.const 2048
            i32.lt_u
            br_if 0 (;@4;)
            block  ;; label = @5
              local.get 1
              i32.const 65536
              i32.lt_u
              br_if 0 (;@5;)
              local.get 2
              local.get 1
              i32.const 63
              i32.and
              i32.const 128
              i32.or
              i32.store8 offset=15
              local.get 2
              local.get 1
              i32.const 18
              i32.shr_u
              i32.const 240
              i32.or
              i32.store8 offset=12
              local.get 2
              local.get 1
              i32.const 6
              i32.shr_u
              i32.const 63
              i32.and
              i32.const 128
              i32.or
              i32.store8 offset=14
              local.get 2
              local.get 1
              i32.const 12
              i32.shr_u
              i32.const 63
              i32.and
              i32.const 128
              i32.or
              i32.store8 offset=13
              i32.const 4
              local.set 1
              br 2 (;@3;)
            end
            local.get 2
            local.get 1
            i32.const 63
            i32.and
            i32.const 128
            i32.or
            i32.store8 offset=14
            local.get 2
            local.get 1
            i32.const 12
            i32.shr_u
            i32.const 224
            i32.or
            i32.store8 offset=12
            local.get 2
            local.get 1
            i32.const 6
            i32.shr_u
            i32.const 63
            i32.and
            i32.const 128
            i32.or
            i32.store8 offset=13
            i32.const 3
            local.set 1
            br 1 (;@3;)
          end
          local.get 2
          local.get 1
          i32.const 63
          i32.and
          i32.const 128
          i32.or
          i32.store8 offset=13
          local.get 2
          local.get 1
          i32.const 6
          i32.shr_u
          i32.const 192
          i32.or
          i32.store8 offset=12
          i32.const 2
          local.set 1
        end
        block  ;; label = @3
          local.get 0
          i32.load
          local.get 0
          i32.load offset=8
          local.tee 3
          i32.sub
          local.get 1
          i32.ge_u
          br_if 0 (;@3;)
          local.get 0
          local.get 3
          local.get 1
          i32.const 1
          i32.const 1
          call $_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17h15fb7f1ec1c750fcE
          local.get 0
          i32.load offset=8
          local.set 3
        end
        local.get 0
        i32.load offset=4
        local.get 3
        i32.add
        local.get 2
        i32.const 12
        i32.add
        local.get 1
        call $memcpy
        drop
        local.get 0
        local.get 3
        local.get 1
        i32.add
        i32.store offset=8
        br 1 (;@1;)
      end
      block  ;; label = @2
        local.get 0
        i32.load offset=8
        local.tee 3
        local.get 0
        i32.load
        i32.ne
        br_if 0 (;@2;)
        local.get 0
        call $_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h6c8115b9fd720c50E
      end
      local.get 0
      local.get 3
      i32.const 1
      i32.add
      i32.store offset=8
      local.get 0
      i32.load offset=4
      local.get 3
      i32.add
      local.get 1
      i32.store8
    end
    local.get 2
    i32.const 16
    i32.add
    global.set $__stack_pointer
    i32.const 0)
  (func $_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h6c8115b9fd720c50E (type 0) (param i32)
    (local i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 0
          i32.load
          local.tee 2
          i32.const -1
          i32.ne
          br_if 0 (;@3;)
          i32.const 0
          local.set 3
          br 1 (;@2;)
        end
        i32.const 0
        local.set 3
        block  ;; label = @3
          local.get 2
          i32.const 1
          i32.shl
          local.tee 4
          local.get 2
          i32.const 1
          i32.add
          local.tee 5
          local.get 4
          local.get 5
          i32.gt_u
          select
          local.tee 4
          i32.const 8
          local.get 4
          i32.const 8
          i32.gt_u
          select
          local.tee 4
          i32.const 0
          i32.ge_s
          br_if 0 (;@3;)
          br 1 (;@2;)
        end
        block  ;; label = @3
          block  ;; label = @4
            local.get 2
            br_if 0 (;@4;)
            i32.const 0
            local.set 2
            br 1 (;@3;)
          end
          local.get 1
          local.get 2
          i32.store offset=28
          local.get 1
          local.get 0
          i32.load offset=4
          i32.store offset=20
          i32.const 1
          local.set 2
        end
        local.get 1
        local.get 2
        i32.store offset=24
        local.get 1
        i32.const 8
        i32.add
        i32.const 1
        local.get 4
        local.get 1
        i32.const 20
        i32.add
        call $_ZN5alloc7raw_vec11finish_grow17h7ff1f438bc526413E
        local.get 1
        i32.load offset=8
        i32.const 1
        i32.ne
        br_if 1 (;@1;)
        local.get 1
        i32.load offset=16
        local.set 0
        local.get 1
        i32.load offset=12
        local.set 3
      end
      local.get 3
      local.get 0
      i32.const 1052952
      call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
      unreachable
    end
    local.get 1
    i32.load offset=12
    local.set 2
    local.get 0
    local.get 4
    i32.store
    local.get 0
    local.get 2
    i32.store offset=4
    local.get 1
    i32.const 32
    i32.add
    global.set $__stack_pointer)
  (func $_ZN58_$LT$alloc..string..String$u20$as$u20$core..fmt..Write$GT$9write_str17habb26b011a335421E (type 6) (param i32 i32 i32) (result i32)
    (local i32)
    block  ;; label = @1
      local.get 0
      i32.load
      local.get 0
      i32.load offset=8
      local.tee 3
      i32.sub
      local.get 2
      i32.ge_u
      br_if 0 (;@1;)
      local.get 0
      local.get 3
      local.get 2
      i32.const 1
      i32.const 1
      call $_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17h15fb7f1ec1c750fcE
      local.get 0
      i32.load offset=8
      local.set 3
    end
    local.get 0
    i32.load offset=4
    local.get 3
    i32.add
    local.get 1
    local.get 2
    call $memcpy
    drop
    local.get 0
    local.get 3
    local.get 2
    i32.add
    i32.store offset=8
    i32.const 0)
  (func $_ZN5alloc7raw_vec11finish_grow17h7ff1f438bc526413E (type 12) (param i32 i32 i32 i32)
    (local i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        i32.const 0
        i32.lt_s
        br_if 0 (;@2;)
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 3
              i32.load offset=4
              i32.eqz
              br_if 0 (;@5;)
              block  ;; label = @6
                local.get 3
                i32.load offset=8
                local.tee 4
                br_if 0 (;@6;)
                block  ;; label = @7
                  local.get 2
                  br_if 0 (;@7;)
                  local.get 1
                  local.set 3
                  br 4 (;@3;)
                end
                i32.const 0
                i32.load8_u offset=1058985
                drop
                br 2 (;@4;)
              end
              local.get 3
              i32.load
              local.get 4
              local.get 1
              local.get 2
              call $__rust_realloc
              local.set 3
              br 2 (;@3;)
            end
            block  ;; label = @5
              local.get 2
              br_if 0 (;@5;)
              local.get 1
              local.set 3
              br 2 (;@3;)
            end
            i32.const 0
            i32.load8_u offset=1058985
            drop
          end
          local.get 2
          local.get 1
          call $__rust_alloc
          local.set 3
        end
        block  ;; label = @3
          local.get 3
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          local.get 2
          i32.store offset=8
          local.get 0
          local.get 3
          i32.store offset=4
          local.get 0
          i32.const 0
          i32.store
          return
        end
        local.get 0
        local.get 2
        i32.store offset=8
        local.get 0
        local.get 1
        i32.store offset=4
        br 1 (;@1;)
      end
      local.get 0
      i32.const 0
      i32.store offset=4
    end
    local.get 0
    i32.const 1
    i32.store)
  (func $_ZN64_$LT$alloc..ffi..c_str..NulError$u20$as$u20$core..fmt..Debug$GT$3fmt17ha182f484657bd2b4E (type 4) (param i32 i32) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    local.get 0
    i32.store offset=12
    local.get 1
    i32.const 1053092
    i32.const 8
    local.get 0
    i32.const 12
    i32.add
    i32.const 1053060
    local.get 2
    i32.const 12
    i32.add
    i32.const 1053076
    call $_ZN4core3fmt9Formatter25debug_tuple_field2_finish17h556ea0cf148a9372E
    local.set 0
    local.get 2
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$12unlink_chunk17h92eb07d772b9ef18E (type 2) (param i32 i32)
    (local i32 i32 i32 i32)
    local.get 0
    i32.load offset=12
    local.set 2
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 1
          i32.const 256
          i32.lt_u
          br_if 0 (;@3;)
          local.get 0
          i32.load offset=24
          local.set 3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                local.get 2
                local.get 0
                i32.ne
                br_if 0 (;@6;)
                local.get 0
                i32.const 20
                i32.const 16
                local.get 0
                i32.load offset=20
                local.tee 2
                select
                i32.add
                i32.load
                local.tee 1
                br_if 1 (;@5;)
                i32.const 0
                local.set 2
                br 2 (;@4;)
              end
              local.get 0
              i32.load offset=8
              local.tee 1
              local.get 2
              i32.store offset=12
              local.get 2
              local.get 1
              i32.store offset=8
              br 1 (;@4;)
            end
            local.get 0
            i32.const 20
            i32.add
            local.get 0
            i32.const 16
            i32.add
            local.get 2
            select
            local.set 4
            loop  ;; label = @5
              local.get 4
              local.set 5
              local.get 1
              local.tee 2
              i32.const 20
              i32.add
              local.get 2
              i32.const 16
              i32.add
              local.get 2
              i32.load offset=20
              local.tee 1
              select
              local.set 4
              local.get 2
              i32.const 20
              i32.const 16
              local.get 1
              select
              i32.add
              i32.load
              local.tee 1
              br_if 0 (;@5;)
            end
            local.get 5
            i32.const 0
            i32.store
          end
          local.get 3
          i32.eqz
          br_if 2 (;@1;)
          block  ;; label = @4
            local.get 0
            i32.load offset=28
            i32.const 2
            i32.shl
            i32.const 1059128
            i32.add
            local.tee 1
            i32.load
            local.get 0
            i32.eq
            br_if 0 (;@4;)
            local.get 3
            i32.const 16
            i32.const 20
            local.get 3
            i32.load offset=16
            local.get 0
            i32.eq
            select
            i32.add
            local.get 2
            i32.store
            local.get 2
            i32.eqz
            br_if 3 (;@1;)
            br 2 (;@2;)
          end
          local.get 1
          local.get 2
          i32.store
          local.get 2
          br_if 1 (;@2;)
          i32.const 0
          i32.const 0
          i32.load offset=1059540
          i32.const -2
          local.get 0
          i32.load offset=28
          i32.rotl
          i32.and
          i32.store offset=1059540
          br 2 (;@1;)
        end
        block  ;; label = @3
          local.get 2
          local.get 0
          i32.load offset=8
          local.tee 4
          i32.eq
          br_if 0 (;@3;)
          local.get 4
          local.get 2
          i32.store offset=12
          local.get 2
          local.get 4
          i32.store offset=8
          return
        end
        i32.const 0
        i32.const 0
        i32.load offset=1059536
        i32.const -2
        local.get 1
        i32.const 3
        i32.shr_u
        i32.rotl
        i32.and
        i32.store offset=1059536
        return
      end
      local.get 2
      local.get 3
      i32.store offset=24
      block  ;; label = @2
        local.get 0
        i32.load offset=16
        local.tee 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 1
        i32.store offset=16
        local.get 1
        local.get 2
        i32.store offset=24
      end
      local.get 0
      i32.load offset=20
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 1
      i32.store offset=20
      local.get 1
      local.get 2
      i32.store offset=24
      return
    end)
  (func $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$13dispose_chunk17h0b59c7a467076600E (type 2) (param i32 i32)
    (local i32 i32)
    local.get 0
    local.get 1
    i32.add
    local.set 2
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.load offset=4
        local.tee 3
        i32.const 1
        i32.and
        br_if 0 (;@2;)
        local.get 3
        i32.const 2
        i32.and
        i32.eqz
        br_if 1 (;@1;)
        local.get 0
        i32.load
        local.tee 3
        local.get 1
        i32.add
        local.set 1
        block  ;; label = @3
          local.get 0
          local.get 3
          i32.sub
          local.tee 0
          i32.const 0
          i32.load offset=1059552
          i32.ne
          br_if 0 (;@3;)
          local.get 2
          i32.load offset=4
          i32.const 3
          i32.and
          i32.const 3
          i32.ne
          br_if 1 (;@2;)
          i32.const 0
          local.get 1
          i32.store offset=1059544
          local.get 2
          local.get 2
          i32.load offset=4
          i32.const -2
          i32.and
          i32.store offset=4
          local.get 0
          local.get 1
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 2
          local.get 1
          i32.store
          br 2 (;@1;)
        end
        local.get 0
        local.get 3
        call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$12unlink_chunk17h92eb07d772b9ef18E
      end
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 2
              i32.load offset=4
              local.tee 3
              i32.const 2
              i32.and
              br_if 0 (;@5;)
              local.get 2
              i32.const 0
              i32.load offset=1059556
              i32.eq
              br_if 2 (;@3;)
              local.get 2
              i32.const 0
              i32.load offset=1059552
              i32.eq
              br_if 3 (;@2;)
              local.get 2
              local.get 3
              i32.const -8
              i32.and
              local.tee 3
              call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$12unlink_chunk17h92eb07d772b9ef18E
              local.get 0
              local.get 3
              local.get 1
              i32.add
              local.tee 1
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 0
              local.get 1
              i32.add
              local.get 1
              i32.store
              local.get 0
              i32.const 0
              i32.load offset=1059552
              i32.ne
              br_if 1 (;@4;)
              i32.const 0
              local.get 1
              i32.store offset=1059544
              return
            end
            local.get 2
            local.get 3
            i32.const -2
            i32.and
            i32.store offset=4
            local.get 0
            local.get 1
            i32.const 1
            i32.or
            i32.store offset=4
            local.get 0
            local.get 1
            i32.add
            local.get 1
            i32.store
          end
          block  ;; label = @4
            local.get 1
            i32.const 256
            i32.lt_u
            br_if 0 (;@4;)
            local.get 0
            local.get 1
            call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$18insert_large_chunk17h368e2b1ffaf0fd0dE
            return
          end
          local.get 1
          i32.const 248
          i32.and
          i32.const 1059272
          i32.add
          local.set 2
          block  ;; label = @4
            block  ;; label = @5
              i32.const 0
              i32.load offset=1059536
              local.tee 3
              i32.const 1
              local.get 1
              i32.const 3
              i32.shr_u
              i32.shl
              local.tee 1
              i32.and
              br_if 0 (;@5;)
              i32.const 0
              local.get 3
              local.get 1
              i32.or
              i32.store offset=1059536
              local.get 2
              local.set 1
              br 1 (;@4;)
            end
            local.get 2
            i32.load offset=8
            local.set 1
          end
          local.get 2
          local.get 0
          i32.store offset=8
          local.get 1
          local.get 0
          i32.store offset=12
          local.get 0
          local.get 2
          i32.store offset=12
          local.get 0
          local.get 1
          i32.store offset=8
          return
        end
        i32.const 0
        local.get 0
        i32.store offset=1059556
        i32.const 0
        i32.const 0
        i32.load offset=1059548
        local.get 1
        i32.add
        local.tee 1
        i32.store offset=1059548
        local.get 0
        local.get 1
        i32.const 1
        i32.or
        i32.store offset=4
        local.get 0
        i32.const 0
        i32.load offset=1059552
        i32.ne
        br_if 1 (;@1;)
        i32.const 0
        i32.const 0
        i32.store offset=1059544
        i32.const 0
        i32.const 0
        i32.store offset=1059552
        return
      end
      i32.const 0
      local.get 0
      i32.store offset=1059552
      i32.const 0
      i32.const 0
      i32.load offset=1059544
      local.get 1
      i32.add
      local.tee 1
      i32.store offset=1059544
      local.get 0
      local.get 1
      i32.const 1
      i32.or
      i32.store offset=4
      local.get 0
      local.get 1
      i32.add
      local.get 1
      i32.store
      return
    end)
  (func $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$18insert_large_chunk17h368e2b1ffaf0fd0dE (type 2) (param i32 i32)
    (local i32 i32 i32 i32)
    i32.const 0
    local.set 2
    block  ;; label = @1
      local.get 1
      i32.const 256
      i32.lt_u
      br_if 0 (;@1;)
      i32.const 31
      local.set 2
      local.get 1
      i32.const 16777215
      i32.gt_u
      br_if 0 (;@1;)
      local.get 1
      i32.const 6
      local.get 1
      i32.const 8
      i32.shr_u
      i32.clz
      local.tee 2
      i32.sub
      i32.shr_u
      i32.const 1
      i32.and
      local.get 2
      i32.const 1
      i32.shl
      i32.sub
      i32.const 62
      i32.add
      local.set 2
    end
    local.get 0
    i64.const 0
    i64.store offset=16 align=4
    local.get 0
    local.get 2
    i32.store offset=28
    local.get 2
    i32.const 2
    i32.shl
    i32.const 1059128
    i32.add
    local.set 3
    block  ;; label = @1
      i32.const 0
      i32.load offset=1059540
      i32.const 1
      local.get 2
      i32.shl
      local.tee 4
      i32.and
      br_if 0 (;@1;)
      local.get 3
      local.get 0
      i32.store
      local.get 0
      local.get 3
      i32.store offset=24
      local.get 0
      local.get 0
      i32.store offset=12
      local.get 0
      local.get 0
      i32.store offset=8
      i32.const 0
      i32.const 0
      i32.load offset=1059540
      local.get 4
      i32.or
      i32.store offset=1059540
      return
    end
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 3
          i32.load
          local.tee 4
          i32.load offset=4
          i32.const -8
          i32.and
          local.get 1
          i32.ne
          br_if 0 (;@3;)
          local.get 4
          local.set 2
          br 1 (;@2;)
        end
        local.get 1
        i32.const 0
        i32.const 25
        local.get 2
        i32.const 1
        i32.shr_u
        i32.sub
        local.get 2
        i32.const 31
        i32.eq
        select
        i32.shl
        local.set 3
        loop  ;; label = @3
          local.get 4
          local.get 3
          i32.const 29
          i32.shr_u
          i32.const 4
          i32.and
          i32.add
          i32.const 16
          i32.add
          local.tee 5
          i32.load
          local.tee 2
          i32.eqz
          br_if 2 (;@1;)
          local.get 3
          i32.const 1
          i32.shl
          local.set 3
          local.get 2
          local.set 4
          local.get 2
          i32.load offset=4
          i32.const -8
          i32.and
          local.get 1
          i32.ne
          br_if 0 (;@3;)
        end
      end
      local.get 2
      i32.load offset=8
      local.tee 3
      local.get 0
      i32.store offset=12
      local.get 2
      local.get 0
      i32.store offset=8
      local.get 0
      i32.const 0
      i32.store offset=24
      local.get 0
      local.get 2
      i32.store offset=12
      local.get 0
      local.get 3
      i32.store offset=8
      return
    end
    local.get 5
    local.get 0
    i32.store
    local.get 0
    local.get 4
    i32.store offset=24
    local.get 0
    local.get 0
    i32.store offset=12
    local.get 0
    local.get 0
    i32.store offset=8)
  (func $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$4free17he5d00a6c001acd28E (type 0) (param i32)
    (local i32 i32 i32 i32 i32)
    local.get 0
    i32.const -8
    i32.add
    local.tee 1
    local.get 0
    i32.const -4
    i32.add
    i32.load
    local.tee 2
    i32.const -8
    i32.and
    local.tee 0
    i32.add
    local.set 3
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        i32.const 1
        i32.and
        br_if 0 (;@2;)
        local.get 2
        i32.const 2
        i32.and
        i32.eqz
        br_if 1 (;@1;)
        local.get 1
        i32.load
        local.tee 2
        local.get 0
        i32.add
        local.set 0
        block  ;; label = @3
          local.get 1
          local.get 2
          i32.sub
          local.tee 1
          i32.const 0
          i32.load offset=1059552
          i32.ne
          br_if 0 (;@3;)
          local.get 3
          i32.load offset=4
          i32.const 3
          i32.and
          i32.const 3
          i32.ne
          br_if 1 (;@2;)
          i32.const 0
          local.get 0
          i32.store offset=1059544
          local.get 3
          local.get 3
          i32.load offset=4
          i32.const -2
          i32.and
          i32.store offset=4
          local.get 1
          local.get 0
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 3
          local.get 0
          i32.store
          return
        end
        local.get 1
        local.get 2
        call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$12unlink_chunk17h92eb07d772b9ef18E
      end
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  local.get 3
                  i32.load offset=4
                  local.tee 2
                  i32.const 2
                  i32.and
                  br_if 0 (;@7;)
                  local.get 3
                  i32.const 0
                  i32.load offset=1059556
                  i32.eq
                  br_if 2 (;@5;)
                  local.get 3
                  i32.const 0
                  i32.load offset=1059552
                  i32.eq
                  br_if 3 (;@4;)
                  local.get 3
                  local.get 2
                  i32.const -8
                  i32.and
                  local.tee 2
                  call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$12unlink_chunk17h92eb07d772b9ef18E
                  local.get 1
                  local.get 2
                  local.get 0
                  i32.add
                  local.tee 0
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  local.get 1
                  local.get 0
                  i32.add
                  local.get 0
                  i32.store
                  local.get 1
                  i32.const 0
                  i32.load offset=1059552
                  i32.ne
                  br_if 1 (;@6;)
                  i32.const 0
                  local.get 0
                  i32.store offset=1059544
                  return
                end
                local.get 3
                local.get 2
                i32.const -2
                i32.and
                i32.store offset=4
                local.get 1
                local.get 0
                i32.const 1
                i32.or
                i32.store offset=4
                local.get 1
                local.get 0
                i32.add
                local.get 0
                i32.store
              end
              local.get 0
              i32.const 256
              i32.lt_u
              br_if 2 (;@3;)
              local.get 1
              local.get 0
              call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$18insert_large_chunk17h368e2b1ffaf0fd0dE
              i32.const 0
              local.set 1
              i32.const 0
              i32.const 0
              i32.load offset=1059576
              i32.const -1
              i32.add
              local.tee 0
              i32.store offset=1059576
              local.get 0
              br_if 4 (;@1;)
              block  ;; label = @6
                i32.const 0
                i32.load offset=1059264
                local.tee 0
                i32.eqz
                br_if 0 (;@6;)
                i32.const 0
                local.set 1
                loop  ;; label = @7
                  local.get 1
                  i32.const 1
                  i32.add
                  local.set 1
                  local.get 0
                  i32.load offset=8
                  local.tee 0
                  br_if 0 (;@7;)
                end
              end
              i32.const 0
              local.get 1
              i32.const 4095
              local.get 1
              i32.const 4095
              i32.gt_u
              select
              i32.store offset=1059576
              return
            end
            i32.const 0
            local.get 1
            i32.store offset=1059556
            i32.const 0
            i32.const 0
            i32.load offset=1059548
            local.get 0
            i32.add
            local.tee 0
            i32.store offset=1059548
            local.get 1
            local.get 0
            i32.const 1
            i32.or
            i32.store offset=4
            block  ;; label = @5
              local.get 1
              i32.const 0
              i32.load offset=1059552
              i32.ne
              br_if 0 (;@5;)
              i32.const 0
              i32.const 0
              i32.store offset=1059544
              i32.const 0
              i32.const 0
              i32.store offset=1059552
            end
            local.get 0
            i32.const 0
            i32.load offset=1059568
            local.tee 4
            i32.le_u
            br_if 3 (;@1;)
            i32.const 0
            i32.load offset=1059556
            local.tee 0
            i32.eqz
            br_if 3 (;@1;)
            i32.const 0
            local.set 2
            i32.const 0
            i32.load offset=1059548
            local.tee 5
            i32.const 41
            i32.lt_u
            br_if 2 (;@2;)
            i32.const 1059256
            local.set 1
            loop  ;; label = @5
              block  ;; label = @6
                local.get 1
                i32.load
                local.tee 3
                local.get 0
                i32.gt_u
                br_if 0 (;@6;)
                local.get 0
                local.get 3
                local.get 1
                i32.load offset=4
                i32.add
                i32.lt_u
                br_if 4 (;@2;)
              end
              local.get 1
              i32.load offset=8
              local.set 1
              br 0 (;@5;)
            end
          end
          i32.const 0
          local.get 1
          i32.store offset=1059552
          i32.const 0
          i32.const 0
          i32.load offset=1059544
          local.get 0
          i32.add
          local.tee 0
          i32.store offset=1059544
          local.get 1
          local.get 0
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 1
          local.get 0
          i32.add
          local.get 0
          i32.store
          return
        end
        local.get 0
        i32.const 248
        i32.and
        i32.const 1059272
        i32.add
        local.set 3
        block  ;; label = @3
          block  ;; label = @4
            i32.const 0
            i32.load offset=1059536
            local.tee 2
            i32.const 1
            local.get 0
            i32.const 3
            i32.shr_u
            i32.shl
            local.tee 0
            i32.and
            br_if 0 (;@4;)
            i32.const 0
            local.get 2
            local.get 0
            i32.or
            i32.store offset=1059536
            local.get 3
            local.set 0
            br 1 (;@3;)
          end
          local.get 3
          i32.load offset=8
          local.set 0
        end
        local.get 3
        local.get 1
        i32.store offset=8
        local.get 0
        local.get 1
        i32.store offset=12
        local.get 1
        local.get 3
        i32.store offset=12
        local.get 1
        local.get 0
        i32.store offset=8
        return
      end
      block  ;; label = @2
        i32.const 0
        i32.load offset=1059264
        local.tee 1
        i32.eqz
        br_if 0 (;@2;)
        i32.const 0
        local.set 2
        loop  ;; label = @3
          local.get 2
          i32.const 1
          i32.add
          local.set 2
          local.get 1
          i32.load offset=8
          local.tee 1
          br_if 0 (;@3;)
        end
      end
      i32.const 0
      local.get 2
      i32.const 4095
      local.get 2
      i32.const 4095
      i32.gt_u
      select
      i32.store offset=1059576
      local.get 5
      local.get 4
      i32.le_u
      br_if 0 (;@1;)
      i32.const 0
      i32.const -1
      i32.store offset=1059568
    end)
  (func $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$6malloc17hf8a67dcfc015198cE (type 5) (param i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i64)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 0
                    i32.const 245
                    i32.lt_u
                    br_if 0 (;@8;)
                    block  ;; label = @9
                      local.get 0
                      i32.const -65587
                      i32.lt_u
                      br_if 0 (;@9;)
                      i32.const 0
                      local.set 0
                      br 8 (;@1;)
                    end
                    local.get 0
                    i32.const 11
                    i32.add
                    local.tee 2
                    i32.const -8
                    i32.and
                    local.set 3
                    i32.const 0
                    i32.load offset=1059540
                    local.tee 4
                    i32.eqz
                    br_if 4 (;@4;)
                    i32.const 31
                    local.set 5
                    block  ;; label = @9
                      local.get 0
                      i32.const 16777204
                      i32.gt_u
                      br_if 0 (;@9;)
                      local.get 3
                      i32.const 6
                      local.get 2
                      i32.const 8
                      i32.shr_u
                      i32.clz
                      local.tee 0
                      i32.sub
                      i32.shr_u
                      i32.const 1
                      i32.and
                      local.get 0
                      i32.const 1
                      i32.shl
                      i32.sub
                      i32.const 62
                      i32.add
                      local.set 5
                    end
                    i32.const 0
                    local.get 3
                    i32.sub
                    local.set 2
                    block  ;; label = @9
                      local.get 5
                      i32.const 2
                      i32.shl
                      i32.const 1059128
                      i32.add
                      i32.load
                      local.tee 6
                      br_if 0 (;@9;)
                      i32.const 0
                      local.set 0
                      i32.const 0
                      local.set 7
                      br 2 (;@7;)
                    end
                    i32.const 0
                    local.set 0
                    local.get 3
                    i32.const 0
                    i32.const 25
                    local.get 5
                    i32.const 1
                    i32.shr_u
                    i32.sub
                    local.get 5
                    i32.const 31
                    i32.eq
                    select
                    i32.shl
                    local.set 8
                    i32.const 0
                    local.set 7
                    loop  ;; label = @9
                      block  ;; label = @10
                        local.get 6
                        local.tee 6
                        i32.load offset=4
                        i32.const -8
                        i32.and
                        local.tee 9
                        local.get 3
                        i32.lt_u
                        br_if 0 (;@10;)
                        local.get 9
                        local.get 3
                        i32.sub
                        local.tee 9
                        local.get 2
                        i32.ge_u
                        br_if 0 (;@10;)
                        local.get 9
                        local.set 2
                        local.get 6
                        local.set 7
                        local.get 9
                        br_if 0 (;@10;)
                        i32.const 0
                        local.set 2
                        local.get 6
                        local.set 7
                        local.get 6
                        local.set 0
                        br 4 (;@6;)
                      end
                      local.get 6
                      i32.load offset=20
                      local.tee 9
                      local.get 0
                      local.get 9
                      local.get 6
                      local.get 8
                      i32.const 29
                      i32.shr_u
                      i32.const 4
                      i32.and
                      i32.add
                      i32.const 16
                      i32.add
                      i32.load
                      local.tee 6
                      i32.ne
                      select
                      local.get 0
                      local.get 9
                      select
                      local.set 0
                      local.get 8
                      i32.const 1
                      i32.shl
                      local.set 8
                      local.get 6
                      i32.eqz
                      br_if 2 (;@7;)
                      br 0 (;@9;)
                    end
                  end
                  block  ;; label = @8
                    i32.const 0
                    i32.load offset=1059536
                    local.tee 6
                    i32.const 16
                    local.get 0
                    i32.const 11
                    i32.add
                    i32.const 504
                    i32.and
                    local.get 0
                    i32.const 11
                    i32.lt_u
                    select
                    local.tee 3
                    i32.const 3
                    i32.shr_u
                    local.tee 2
                    i32.shr_u
                    local.tee 0
                    i32.const 3
                    i32.and
                    i32.eqz
                    br_if 0 (;@8;)
                    block  ;; label = @9
                      block  ;; label = @10
                        local.get 0
                        i32.const -1
                        i32.xor
                        i32.const 1
                        i32.and
                        local.get 2
                        i32.add
                        local.tee 8
                        i32.const 3
                        i32.shl
                        local.tee 3
                        i32.const 1059272
                        i32.add
                        local.tee 0
                        local.get 3
                        i32.const 1059280
                        i32.add
                        i32.load
                        local.tee 2
                        i32.load offset=8
                        local.tee 7
                        i32.eq
                        br_if 0 (;@10;)
                        local.get 7
                        local.get 0
                        i32.store offset=12
                        local.get 0
                        local.get 7
                        i32.store offset=8
                        br 1 (;@9;)
                      end
                      i32.const 0
                      local.get 6
                      i32.const -2
                      local.get 8
                      i32.rotl
                      i32.and
                      i32.store offset=1059536
                    end
                    local.get 2
                    i32.const 8
                    i32.add
                    local.set 0
                    local.get 2
                    local.get 3
                    i32.const 3
                    i32.or
                    i32.store offset=4
                    local.get 2
                    local.get 3
                    i32.add
                    local.tee 3
                    local.get 3
                    i32.load offset=4
                    i32.const 1
                    i32.or
                    i32.store offset=4
                    br 7 (;@1;)
                  end
                  local.get 3
                  i32.const 0
                  i32.load offset=1059544
                  i32.le_u
                  br_if 3 (;@4;)
                  block  ;; label = @8
                    block  ;; label = @9
                      block  ;; label = @10
                        local.get 0
                        br_if 0 (;@10;)
                        i32.const 0
                        i32.load offset=1059540
                        local.tee 0
                        i32.eqz
                        br_if 6 (;@4;)
                        local.get 0
                        i32.ctz
                        i32.const 2
                        i32.shl
                        i32.const 1059128
                        i32.add
                        i32.load
                        local.tee 7
                        i32.load offset=4
                        i32.const -8
                        i32.and
                        local.get 3
                        i32.sub
                        local.set 2
                        local.get 7
                        local.set 6
                        loop  ;; label = @11
                          block  ;; label = @12
                            local.get 7
                            i32.load offset=16
                            local.tee 0
                            br_if 0 (;@12;)
                            local.get 7
                            i32.load offset=20
                            local.tee 0
                            br_if 0 (;@12;)
                            local.get 6
                            i32.load offset=24
                            local.set 5
                            block  ;; label = @13
                              block  ;; label = @14
                                block  ;; label = @15
                                  local.get 6
                                  i32.load offset=12
                                  local.tee 0
                                  local.get 6
                                  i32.ne
                                  br_if 0 (;@15;)
                                  local.get 6
                                  i32.const 20
                                  i32.const 16
                                  local.get 6
                                  i32.load offset=20
                                  local.tee 0
                                  select
                                  i32.add
                                  i32.load
                                  local.tee 7
                                  br_if 1 (;@14;)
                                  i32.const 0
                                  local.set 0
                                  br 2 (;@13;)
                                end
                                local.get 6
                                i32.load offset=8
                                local.tee 7
                                local.get 0
                                i32.store offset=12
                                local.get 0
                                local.get 7
                                i32.store offset=8
                                br 1 (;@13;)
                              end
                              local.get 6
                              i32.const 20
                              i32.add
                              local.get 6
                              i32.const 16
                              i32.add
                              local.get 0
                              select
                              local.set 8
                              loop  ;; label = @14
                                local.get 8
                                local.set 9
                                local.get 7
                                local.tee 0
                                i32.const 20
                                i32.add
                                local.get 0
                                i32.const 16
                                i32.add
                                local.get 0
                                i32.load offset=20
                                local.tee 7
                                select
                                local.set 8
                                local.get 0
                                i32.const 20
                                i32.const 16
                                local.get 7
                                select
                                i32.add
                                i32.load
                                local.tee 7
                                br_if 0 (;@14;)
                              end
                              local.get 9
                              i32.const 0
                              i32.store
                            end
                            local.get 5
                            i32.eqz
                            br_if 4 (;@8;)
                            block  ;; label = @13
                              local.get 6
                              i32.load offset=28
                              i32.const 2
                              i32.shl
                              i32.const 1059128
                              i32.add
                              local.tee 7
                              i32.load
                              local.get 6
                              i32.eq
                              br_if 0 (;@13;)
                              local.get 5
                              i32.const 16
                              i32.const 20
                              local.get 5
                              i32.load offset=16
                              local.get 6
                              i32.eq
                              select
                              i32.add
                              local.get 0
                              i32.store
                              local.get 0
                              i32.eqz
                              br_if 5 (;@8;)
                              br 4 (;@9;)
                            end
                            local.get 7
                            local.get 0
                            i32.store
                            local.get 0
                            br_if 3 (;@9;)
                            i32.const 0
                            i32.const 0
                            i32.load offset=1059540
                            i32.const -2
                            local.get 6
                            i32.load offset=28
                            i32.rotl
                            i32.and
                            i32.store offset=1059540
                            br 4 (;@8;)
                          end
                          local.get 0
                          i32.load offset=4
                          i32.const -8
                          i32.and
                          local.get 3
                          i32.sub
                          local.tee 7
                          local.get 2
                          local.get 7
                          local.get 2
                          i32.lt_u
                          local.tee 7
                          select
                          local.set 2
                          local.get 0
                          local.get 6
                          local.get 7
                          select
                          local.set 6
                          local.get 0
                          local.set 7
                          br 0 (;@11;)
                        end
                      end
                      block  ;; label = @10
                        block  ;; label = @11
                          local.get 0
                          local.get 2
                          i32.shl
                          i32.const 2
                          local.get 2
                          i32.shl
                          local.tee 0
                          i32.const 0
                          local.get 0
                          i32.sub
                          i32.or
                          i32.and
                          i32.ctz
                          local.tee 9
                          i32.const 3
                          i32.shl
                          local.tee 2
                          i32.const 1059272
                          i32.add
                          local.tee 7
                          local.get 2
                          i32.const 1059280
                          i32.add
                          i32.load
                          local.tee 0
                          i32.load offset=8
                          local.tee 8
                          i32.eq
                          br_if 0 (;@11;)
                          local.get 8
                          local.get 7
                          i32.store offset=12
                          local.get 7
                          local.get 8
                          i32.store offset=8
                          br 1 (;@10;)
                        end
                        i32.const 0
                        local.get 6
                        i32.const -2
                        local.get 9
                        i32.rotl
                        i32.and
                        i32.store offset=1059536
                      end
                      local.get 0
                      local.get 3
                      i32.const 3
                      i32.or
                      i32.store offset=4
                      local.get 0
                      local.get 3
                      i32.add
                      local.tee 8
                      local.get 2
                      local.get 3
                      i32.sub
                      local.tee 7
                      i32.const 1
                      i32.or
                      i32.store offset=4
                      local.get 0
                      local.get 2
                      i32.add
                      local.get 7
                      i32.store
                      block  ;; label = @10
                        i32.const 0
                        i32.load offset=1059544
                        local.tee 6
                        i32.eqz
                        br_if 0 (;@10;)
                        local.get 6
                        i32.const -8
                        i32.and
                        i32.const 1059272
                        i32.add
                        local.set 2
                        i32.const 0
                        i32.load offset=1059552
                        local.set 3
                        block  ;; label = @11
                          block  ;; label = @12
                            i32.const 0
                            i32.load offset=1059536
                            local.tee 9
                            i32.const 1
                            local.get 6
                            i32.const 3
                            i32.shr_u
                            i32.shl
                            local.tee 6
                            i32.and
                            br_if 0 (;@12;)
                            i32.const 0
                            local.get 9
                            local.get 6
                            i32.or
                            i32.store offset=1059536
                            local.get 2
                            local.set 6
                            br 1 (;@11;)
                          end
                          local.get 2
                          i32.load offset=8
                          local.set 6
                        end
                        local.get 2
                        local.get 3
                        i32.store offset=8
                        local.get 6
                        local.get 3
                        i32.store offset=12
                        local.get 3
                        local.get 2
                        i32.store offset=12
                        local.get 3
                        local.get 6
                        i32.store offset=8
                      end
                      local.get 0
                      i32.const 8
                      i32.add
                      local.set 0
                      i32.const 0
                      local.get 8
                      i32.store offset=1059552
                      i32.const 0
                      local.get 7
                      i32.store offset=1059544
                      br 8 (;@1;)
                    end
                    local.get 0
                    local.get 5
                    i32.store offset=24
                    block  ;; label = @9
                      local.get 6
                      i32.load offset=16
                      local.tee 7
                      i32.eqz
                      br_if 0 (;@9;)
                      local.get 0
                      local.get 7
                      i32.store offset=16
                      local.get 7
                      local.get 0
                      i32.store offset=24
                    end
                    local.get 6
                    i32.load offset=20
                    local.tee 7
                    i32.eqz
                    br_if 0 (;@8;)
                    local.get 0
                    local.get 7
                    i32.store offset=20
                    local.get 7
                    local.get 0
                    i32.store offset=24
                  end
                  block  ;; label = @8
                    block  ;; label = @9
                      block  ;; label = @10
                        local.get 2
                        i32.const 16
                        i32.lt_u
                        br_if 0 (;@10;)
                        local.get 6
                        local.get 3
                        i32.const 3
                        i32.or
                        i32.store offset=4
                        local.get 6
                        local.get 3
                        i32.add
                        local.tee 3
                        local.get 2
                        i32.const 1
                        i32.or
                        i32.store offset=4
                        local.get 3
                        local.get 2
                        i32.add
                        local.get 2
                        i32.store
                        i32.const 0
                        i32.load offset=1059544
                        local.tee 8
                        i32.eqz
                        br_if 1 (;@9;)
                        local.get 8
                        i32.const -8
                        i32.and
                        i32.const 1059272
                        i32.add
                        local.set 7
                        i32.const 0
                        i32.load offset=1059552
                        local.set 0
                        block  ;; label = @11
                          block  ;; label = @12
                            i32.const 0
                            i32.load offset=1059536
                            local.tee 9
                            i32.const 1
                            local.get 8
                            i32.const 3
                            i32.shr_u
                            i32.shl
                            local.tee 8
                            i32.and
                            br_if 0 (;@12;)
                            i32.const 0
                            local.get 9
                            local.get 8
                            i32.or
                            i32.store offset=1059536
                            local.get 7
                            local.set 8
                            br 1 (;@11;)
                          end
                          local.get 7
                          i32.load offset=8
                          local.set 8
                        end
                        local.get 7
                        local.get 0
                        i32.store offset=8
                        local.get 8
                        local.get 0
                        i32.store offset=12
                        local.get 0
                        local.get 7
                        i32.store offset=12
                        local.get 0
                        local.get 8
                        i32.store offset=8
                        br 1 (;@9;)
                      end
                      local.get 6
                      local.get 2
                      local.get 3
                      i32.add
                      local.tee 0
                      i32.const 3
                      i32.or
                      i32.store offset=4
                      local.get 6
                      local.get 0
                      i32.add
                      local.tee 0
                      local.get 0
                      i32.load offset=4
                      i32.const 1
                      i32.or
                      i32.store offset=4
                      br 1 (;@8;)
                    end
                    i32.const 0
                    local.get 3
                    i32.store offset=1059552
                    i32.const 0
                    local.get 2
                    i32.store offset=1059544
                  end
                  local.get 6
                  i32.const 8
                  i32.add
                  local.set 0
                  br 6 (;@1;)
                end
                block  ;; label = @7
                  local.get 0
                  local.get 7
                  i32.or
                  br_if 0 (;@7;)
                  i32.const 0
                  local.set 7
                  i32.const 2
                  local.get 5
                  i32.shl
                  local.tee 0
                  i32.const 0
                  local.get 0
                  i32.sub
                  i32.or
                  local.get 4
                  i32.and
                  local.tee 0
                  i32.eqz
                  br_if 3 (;@4;)
                  local.get 0
                  i32.ctz
                  i32.const 2
                  i32.shl
                  i32.const 1059128
                  i32.add
                  i32.load
                  local.set 0
                end
                local.get 0
                i32.eqz
                br_if 1 (;@5;)
              end
              loop  ;; label = @6
                local.get 0
                local.get 7
                local.get 0
                i32.load offset=4
                i32.const -8
                i32.and
                local.tee 6
                local.get 3
                i32.sub
                local.tee 9
                local.get 2
                i32.lt_u
                local.tee 5
                select
                local.set 4
                local.get 6
                local.get 3
                i32.lt_u
                local.set 8
                local.get 9
                local.get 2
                local.get 5
                select
                local.set 9
                block  ;; label = @7
                  local.get 0
                  i32.load offset=16
                  local.tee 6
                  br_if 0 (;@7;)
                  local.get 0
                  i32.load offset=20
                  local.set 6
                end
                local.get 7
                local.get 4
                local.get 8
                select
                local.set 7
                local.get 2
                local.get 9
                local.get 8
                select
                local.set 2
                local.get 6
                local.set 0
                local.get 6
                br_if 0 (;@6;)
              end
            end
            local.get 7
            i32.eqz
            br_if 0 (;@4;)
            block  ;; label = @5
              i32.const 0
              i32.load offset=1059544
              local.tee 0
              local.get 3
              i32.lt_u
              br_if 0 (;@5;)
              local.get 2
              local.get 0
              local.get 3
              i32.sub
              i32.ge_u
              br_if 1 (;@4;)
            end
            local.get 7
            i32.load offset=24
            local.set 5
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  local.get 7
                  i32.load offset=12
                  local.tee 0
                  local.get 7
                  i32.ne
                  br_if 0 (;@7;)
                  local.get 7
                  i32.const 20
                  i32.const 16
                  local.get 7
                  i32.load offset=20
                  local.tee 0
                  select
                  i32.add
                  i32.load
                  local.tee 6
                  br_if 1 (;@6;)
                  i32.const 0
                  local.set 0
                  br 2 (;@5;)
                end
                local.get 7
                i32.load offset=8
                local.tee 6
                local.get 0
                i32.store offset=12
                local.get 0
                local.get 6
                i32.store offset=8
                br 1 (;@5;)
              end
              local.get 7
              i32.const 20
              i32.add
              local.get 7
              i32.const 16
              i32.add
              local.get 0
              select
              local.set 8
              loop  ;; label = @6
                local.get 8
                local.set 9
                local.get 6
                local.tee 0
                i32.const 20
                i32.add
                local.get 0
                i32.const 16
                i32.add
                local.get 0
                i32.load offset=20
                local.tee 6
                select
                local.set 8
                local.get 0
                i32.const 20
                i32.const 16
                local.get 6
                select
                i32.add
                i32.load
                local.tee 6
                br_if 0 (;@6;)
              end
              local.get 9
              i32.const 0
              i32.store
            end
            local.get 5
            i32.eqz
            br_if 2 (;@2;)
            block  ;; label = @5
              local.get 7
              i32.load offset=28
              i32.const 2
              i32.shl
              i32.const 1059128
              i32.add
              local.tee 6
              i32.load
              local.get 7
              i32.eq
              br_if 0 (;@5;)
              local.get 5
              i32.const 16
              i32.const 20
              local.get 5
              i32.load offset=16
              local.get 7
              i32.eq
              select
              i32.add
              local.get 0
              i32.store
              local.get 0
              i32.eqz
              br_if 3 (;@2;)
              br 2 (;@3;)
            end
            local.get 6
            local.get 0
            i32.store
            local.get 0
            br_if 1 (;@3;)
            i32.const 0
            i32.const 0
            i32.load offset=1059540
            i32.const -2
            local.get 7
            i32.load offset=28
            i32.rotl
            i32.and
            i32.store offset=1059540
            br 2 (;@2;)
          end
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      i32.const 0
                      i32.load offset=1059544
                      local.tee 0
                      local.get 3
                      i32.ge_u
                      br_if 0 (;@9;)
                      block  ;; label = @10
                        i32.const 0
                        i32.load offset=1059548
                        local.tee 0
                        local.get 3
                        i32.gt_u
                        br_if 0 (;@10;)
                        local.get 1
                        i32.const 4
                        i32.add
                        i32.const 1059580
                        local.get 3
                        i32.const 65583
                        i32.add
                        i32.const -65536
                        i32.and
                        call $_ZN61_$LT$dlmalloc..sys..System$u20$as$u20$dlmalloc..Allocator$GT$5alloc17hd432c065eb8119e7E
                        block  ;; label = @11
                          local.get 1
                          i32.load offset=4
                          local.tee 6
                          br_if 0 (;@11;)
                          i32.const 0
                          local.set 0
                          br 10 (;@1;)
                        end
                        local.get 1
                        i32.load offset=12
                        local.set 5
                        i32.const 0
                        i32.const 0
                        i32.load offset=1059560
                        local.get 1
                        i32.load offset=8
                        local.tee 9
                        i32.add
                        local.tee 0
                        i32.store offset=1059560
                        i32.const 0
                        i32.const 0
                        i32.load offset=1059564
                        local.tee 2
                        local.get 0
                        local.get 2
                        local.get 0
                        i32.gt_u
                        select
                        i32.store offset=1059564
                        block  ;; label = @11
                          block  ;; label = @12
                            block  ;; label = @13
                              i32.const 0
                              i32.load offset=1059556
                              local.tee 2
                              i32.eqz
                              br_if 0 (;@13;)
                              i32.const 1059256
                              local.set 0
                              loop  ;; label = @14
                                local.get 6
                                local.get 0
                                i32.load
                                local.tee 7
                                local.get 0
                                i32.load offset=4
                                local.tee 8
                                i32.add
                                i32.eq
                                br_if 2 (;@12;)
                                local.get 0
                                i32.load offset=8
                                local.tee 0
                                br_if 0 (;@14;)
                                br 3 (;@11;)
                              end
                            end
                            block  ;; label = @13
                              block  ;; label = @14
                                i32.const 0
                                i32.load offset=1059572
                                local.tee 0
                                i32.eqz
                                br_if 0 (;@14;)
                                local.get 6
                                local.get 0
                                i32.ge_u
                                br_if 1 (;@13;)
                              end
                              i32.const 0
                              local.get 6
                              i32.store offset=1059572
                            end
                            i32.const 0
                            i32.const 4095
                            i32.store offset=1059576
                            i32.const 0
                            local.get 5
                            i32.store offset=1059268
                            i32.const 0
                            local.get 9
                            i32.store offset=1059260
                            i32.const 0
                            local.get 6
                            i32.store offset=1059256
                            i32.const 0
                            i32.const 1059272
                            i32.store offset=1059284
                            i32.const 0
                            i32.const 1059280
                            i32.store offset=1059292
                            i32.const 0
                            i32.const 1059272
                            i32.store offset=1059280
                            i32.const 0
                            i32.const 1059288
                            i32.store offset=1059300
                            i32.const 0
                            i32.const 1059280
                            i32.store offset=1059288
                            i32.const 0
                            i32.const 1059296
                            i32.store offset=1059308
                            i32.const 0
                            i32.const 1059288
                            i32.store offset=1059296
                            i32.const 0
                            i32.const 1059304
                            i32.store offset=1059316
                            i32.const 0
                            i32.const 1059296
                            i32.store offset=1059304
                            i32.const 0
                            i32.const 1059312
                            i32.store offset=1059324
                            i32.const 0
                            i32.const 1059304
                            i32.store offset=1059312
                            i32.const 0
                            i32.const 1059320
                            i32.store offset=1059332
                            i32.const 0
                            i32.const 1059312
                            i32.store offset=1059320
                            i32.const 0
                            i32.const 1059328
                            i32.store offset=1059340
                            i32.const 0
                            i32.const 1059320
                            i32.store offset=1059328
                            i32.const 0
                            i32.const 1059336
                            i32.store offset=1059348
                            i32.const 0
                            i32.const 1059328
                            i32.store offset=1059336
                            i32.const 0
                            i32.const 1059336
                            i32.store offset=1059344
                            i32.const 0
                            i32.const 1059344
                            i32.store offset=1059356
                            i32.const 0
                            i32.const 1059344
                            i32.store offset=1059352
                            i32.const 0
                            i32.const 1059352
                            i32.store offset=1059364
                            i32.const 0
                            i32.const 1059352
                            i32.store offset=1059360
                            i32.const 0
                            i32.const 1059360
                            i32.store offset=1059372
                            i32.const 0
                            i32.const 1059360
                            i32.store offset=1059368
                            i32.const 0
                            i32.const 1059368
                            i32.store offset=1059380
                            i32.const 0
                            i32.const 1059368
                            i32.store offset=1059376
                            i32.const 0
                            i32.const 1059376
                            i32.store offset=1059388
                            i32.const 0
                            i32.const 1059376
                            i32.store offset=1059384
                            i32.const 0
                            i32.const 1059384
                            i32.store offset=1059396
                            i32.const 0
                            i32.const 1059384
                            i32.store offset=1059392
                            i32.const 0
                            i32.const 1059392
                            i32.store offset=1059404
                            i32.const 0
                            i32.const 1059392
                            i32.store offset=1059400
                            i32.const 0
                            i32.const 1059400
                            i32.store offset=1059412
                            i32.const 0
                            i32.const 1059408
                            i32.store offset=1059420
                            i32.const 0
                            i32.const 1059400
                            i32.store offset=1059408
                            i32.const 0
                            i32.const 1059416
                            i32.store offset=1059428
                            i32.const 0
                            i32.const 1059408
                            i32.store offset=1059416
                            i32.const 0
                            i32.const 1059424
                            i32.store offset=1059436
                            i32.const 0
                            i32.const 1059416
                            i32.store offset=1059424
                            i32.const 0
                            i32.const 1059432
                            i32.store offset=1059444
                            i32.const 0
                            i32.const 1059424
                            i32.store offset=1059432
                            i32.const 0
                            i32.const 1059440
                            i32.store offset=1059452
                            i32.const 0
                            i32.const 1059432
                            i32.store offset=1059440
                            i32.const 0
                            i32.const 1059448
                            i32.store offset=1059460
                            i32.const 0
                            i32.const 1059440
                            i32.store offset=1059448
                            i32.const 0
                            i32.const 1059456
                            i32.store offset=1059468
                            i32.const 0
                            i32.const 1059448
                            i32.store offset=1059456
                            i32.const 0
                            i32.const 1059464
                            i32.store offset=1059476
                            i32.const 0
                            i32.const 1059456
                            i32.store offset=1059464
                            i32.const 0
                            i32.const 1059472
                            i32.store offset=1059484
                            i32.const 0
                            i32.const 1059464
                            i32.store offset=1059472
                            i32.const 0
                            i32.const 1059480
                            i32.store offset=1059492
                            i32.const 0
                            i32.const 1059472
                            i32.store offset=1059480
                            i32.const 0
                            i32.const 1059488
                            i32.store offset=1059500
                            i32.const 0
                            i32.const 1059480
                            i32.store offset=1059488
                            i32.const 0
                            i32.const 1059496
                            i32.store offset=1059508
                            i32.const 0
                            i32.const 1059488
                            i32.store offset=1059496
                            i32.const 0
                            i32.const 1059504
                            i32.store offset=1059516
                            i32.const 0
                            i32.const 1059496
                            i32.store offset=1059504
                            i32.const 0
                            i32.const 1059512
                            i32.store offset=1059524
                            i32.const 0
                            i32.const 1059504
                            i32.store offset=1059512
                            i32.const 0
                            i32.const 1059520
                            i32.store offset=1059532
                            i32.const 0
                            i32.const 1059512
                            i32.store offset=1059520
                            i32.const 0
                            local.get 6
                            i32.const 15
                            i32.add
                            i32.const -8
                            i32.and
                            local.tee 0
                            i32.const -8
                            i32.add
                            local.tee 2
                            i32.store offset=1059556
                            i32.const 0
                            i32.const 1059520
                            i32.store offset=1059528
                            i32.const 0
                            local.get 6
                            local.get 0
                            i32.sub
                            local.get 9
                            i32.const -40
                            i32.add
                            local.tee 0
                            i32.add
                            i32.const 8
                            i32.add
                            local.tee 7
                            i32.store offset=1059548
                            local.get 2
                            local.get 7
                            i32.const 1
                            i32.or
                            i32.store offset=4
                            local.get 6
                            local.get 0
                            i32.add
                            i32.const 40
                            i32.store offset=4
                            i32.const 0
                            i32.const 2097152
                            i32.store offset=1059568
                            br 8 (;@4;)
                          end
                          local.get 2
                          local.get 6
                          i32.ge_u
                          br_if 0 (;@11;)
                          local.get 7
                          local.get 2
                          i32.gt_u
                          br_if 0 (;@11;)
                          local.get 0
                          i32.load offset=12
                          local.tee 7
                          i32.const 1
                          i32.and
                          br_if 0 (;@11;)
                          local.get 7
                          i32.const 1
                          i32.shr_u
                          local.get 5
                          i32.eq
                          br_if 3 (;@8;)
                        end
                        i32.const 0
                        i32.const 0
                        i32.load offset=1059572
                        local.tee 0
                        local.get 6
                        local.get 6
                        local.get 0
                        i32.gt_u
                        select
                        i32.store offset=1059572
                        local.get 6
                        local.get 9
                        i32.add
                        local.set 7
                        i32.const 1059256
                        local.set 0
                        block  ;; label = @11
                          block  ;; label = @12
                            block  ;; label = @13
                              loop  ;; label = @14
                                local.get 0
                                i32.load
                                local.tee 8
                                local.get 7
                                i32.eq
                                br_if 1 (;@13;)
                                local.get 0
                                i32.load offset=8
                                local.tee 0
                                br_if 0 (;@14;)
                                br 2 (;@12;)
                              end
                            end
                            local.get 0
                            i32.load offset=12
                            local.tee 7
                            i32.const 1
                            i32.and
                            br_if 0 (;@12;)
                            local.get 7
                            i32.const 1
                            i32.shr_u
                            local.get 5
                            i32.eq
                            br_if 1 (;@11;)
                          end
                          i32.const 1059256
                          local.set 0
                          block  ;; label = @12
                            loop  ;; label = @13
                              block  ;; label = @14
                                local.get 0
                                i32.load
                                local.tee 7
                                local.get 2
                                i32.gt_u
                                br_if 0 (;@14;)
                                local.get 2
                                local.get 7
                                local.get 0
                                i32.load offset=4
                                i32.add
                                local.tee 7
                                i32.lt_u
                                br_if 2 (;@12;)
                              end
                              local.get 0
                              i32.load offset=8
                              local.set 0
                              br 0 (;@13;)
                            end
                          end
                          i32.const 0
                          local.get 6
                          i32.const 15
                          i32.add
                          i32.const -8
                          i32.and
                          local.tee 0
                          i32.const -8
                          i32.add
                          local.tee 8
                          i32.store offset=1059556
                          i32.const 0
                          local.get 6
                          local.get 0
                          i32.sub
                          local.get 9
                          i32.const -40
                          i32.add
                          local.tee 0
                          i32.add
                          i32.const 8
                          i32.add
                          local.tee 4
                          i32.store offset=1059548
                          local.get 8
                          local.get 4
                          i32.const 1
                          i32.or
                          i32.store offset=4
                          local.get 6
                          local.get 0
                          i32.add
                          i32.const 40
                          i32.store offset=4
                          i32.const 0
                          i32.const 2097152
                          i32.store offset=1059568
                          local.get 2
                          local.get 7
                          i32.const -32
                          i32.add
                          i32.const -8
                          i32.and
                          i32.const -8
                          i32.add
                          local.tee 0
                          local.get 0
                          local.get 2
                          i32.const 16
                          i32.add
                          i32.lt_u
                          select
                          local.tee 8
                          i32.const 27
                          i32.store offset=4
                          i32.const 0
                          i64.load offset=1059256 align=4
                          local.set 10
                          local.get 8
                          i32.const 16
                          i32.add
                          i32.const 0
                          i64.load offset=1059264 align=4
                          i64.store align=4
                          local.get 8
                          local.get 10
                          i64.store offset=8 align=4
                          i32.const 0
                          local.get 5
                          i32.store offset=1059268
                          i32.const 0
                          local.get 9
                          i32.store offset=1059260
                          i32.const 0
                          local.get 6
                          i32.store offset=1059256
                          i32.const 0
                          local.get 8
                          i32.const 8
                          i32.add
                          i32.store offset=1059264
                          local.get 8
                          i32.const 28
                          i32.add
                          local.set 0
                          loop  ;; label = @12
                            local.get 0
                            i32.const 7
                            i32.store
                            local.get 0
                            i32.const 4
                            i32.add
                            local.tee 0
                            local.get 7
                            i32.lt_u
                            br_if 0 (;@12;)
                          end
                          local.get 8
                          local.get 2
                          i32.eq
                          br_if 7 (;@4;)
                          local.get 8
                          local.get 8
                          i32.load offset=4
                          i32.const -2
                          i32.and
                          i32.store offset=4
                          local.get 2
                          local.get 8
                          local.get 2
                          i32.sub
                          local.tee 0
                          i32.const 1
                          i32.or
                          i32.store offset=4
                          local.get 8
                          local.get 0
                          i32.store
                          block  ;; label = @12
                            local.get 0
                            i32.const 256
                            i32.lt_u
                            br_if 0 (;@12;)
                            local.get 2
                            local.get 0
                            call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$18insert_large_chunk17h368e2b1ffaf0fd0dE
                            br 8 (;@4;)
                          end
                          local.get 0
                          i32.const 248
                          i32.and
                          i32.const 1059272
                          i32.add
                          local.set 7
                          block  ;; label = @12
                            block  ;; label = @13
                              i32.const 0
                              i32.load offset=1059536
                              local.tee 6
                              i32.const 1
                              local.get 0
                              i32.const 3
                              i32.shr_u
                              i32.shl
                              local.tee 0
                              i32.and
                              br_if 0 (;@13;)
                              i32.const 0
                              local.get 6
                              local.get 0
                              i32.or
                              i32.store offset=1059536
                              local.get 7
                              local.set 0
                              br 1 (;@12;)
                            end
                            local.get 7
                            i32.load offset=8
                            local.set 0
                          end
                          local.get 7
                          local.get 2
                          i32.store offset=8
                          local.get 0
                          local.get 2
                          i32.store offset=12
                          local.get 2
                          local.get 7
                          i32.store offset=12
                          local.get 2
                          local.get 0
                          i32.store offset=8
                          br 7 (;@4;)
                        end
                        local.get 0
                        local.get 6
                        i32.store
                        local.get 0
                        local.get 0
                        i32.load offset=4
                        local.get 9
                        i32.add
                        i32.store offset=4
                        local.get 6
                        i32.const 15
                        i32.add
                        i32.const -8
                        i32.and
                        i32.const -8
                        i32.add
                        local.tee 7
                        local.get 3
                        i32.const 3
                        i32.or
                        i32.store offset=4
                        local.get 8
                        i32.const 15
                        i32.add
                        i32.const -8
                        i32.and
                        i32.const -8
                        i32.add
                        local.tee 2
                        local.get 7
                        local.get 3
                        i32.add
                        local.tee 0
                        i32.sub
                        local.set 3
                        local.get 2
                        i32.const 0
                        i32.load offset=1059556
                        i32.eq
                        br_if 3 (;@7;)
                        local.get 2
                        i32.const 0
                        i32.load offset=1059552
                        i32.eq
                        br_if 4 (;@6;)
                        block  ;; label = @11
                          local.get 2
                          i32.load offset=4
                          local.tee 6
                          i32.const 3
                          i32.and
                          i32.const 1
                          i32.ne
                          br_if 0 (;@11;)
                          local.get 2
                          local.get 6
                          i32.const -8
                          i32.and
                          local.tee 6
                          call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$12unlink_chunk17h92eb07d772b9ef18E
                          local.get 6
                          local.get 3
                          i32.add
                          local.set 3
                          local.get 2
                          local.get 6
                          i32.add
                          local.tee 2
                          i32.load offset=4
                          local.set 6
                        end
                        local.get 2
                        local.get 6
                        i32.const -2
                        i32.and
                        i32.store offset=4
                        local.get 0
                        local.get 3
                        i32.const 1
                        i32.or
                        i32.store offset=4
                        local.get 0
                        local.get 3
                        i32.add
                        local.get 3
                        i32.store
                        block  ;; label = @11
                          local.get 3
                          i32.const 256
                          i32.lt_u
                          br_if 0 (;@11;)
                          local.get 0
                          local.get 3
                          call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$18insert_large_chunk17h368e2b1ffaf0fd0dE
                          br 6 (;@5;)
                        end
                        local.get 3
                        i32.const 248
                        i32.and
                        i32.const 1059272
                        i32.add
                        local.set 2
                        block  ;; label = @11
                          block  ;; label = @12
                            i32.const 0
                            i32.load offset=1059536
                            local.tee 6
                            i32.const 1
                            local.get 3
                            i32.const 3
                            i32.shr_u
                            i32.shl
                            local.tee 3
                            i32.and
                            br_if 0 (;@12;)
                            i32.const 0
                            local.get 6
                            local.get 3
                            i32.or
                            i32.store offset=1059536
                            local.get 2
                            local.set 3
                            br 1 (;@11;)
                          end
                          local.get 2
                          i32.load offset=8
                          local.set 3
                        end
                        local.get 2
                        local.get 0
                        i32.store offset=8
                        local.get 3
                        local.get 0
                        i32.store offset=12
                        local.get 0
                        local.get 2
                        i32.store offset=12
                        local.get 0
                        local.get 3
                        i32.store offset=8
                        br 5 (;@5;)
                      end
                      i32.const 0
                      local.get 0
                      local.get 3
                      i32.sub
                      local.tee 2
                      i32.store offset=1059548
                      i32.const 0
                      i32.const 0
                      i32.load offset=1059556
                      local.tee 0
                      local.get 3
                      i32.add
                      local.tee 7
                      i32.store offset=1059556
                      local.get 7
                      local.get 2
                      i32.const 1
                      i32.or
                      i32.store offset=4
                      local.get 0
                      local.get 3
                      i32.const 3
                      i32.or
                      i32.store offset=4
                      local.get 0
                      i32.const 8
                      i32.add
                      local.set 0
                      br 8 (;@1;)
                    end
                    i32.const 0
                    i32.load offset=1059552
                    local.set 2
                    block  ;; label = @9
                      block  ;; label = @10
                        local.get 0
                        local.get 3
                        i32.sub
                        local.tee 7
                        i32.const 15
                        i32.gt_u
                        br_if 0 (;@10;)
                        i32.const 0
                        i32.const 0
                        i32.store offset=1059552
                        i32.const 0
                        i32.const 0
                        i32.store offset=1059544
                        local.get 2
                        local.get 0
                        i32.const 3
                        i32.or
                        i32.store offset=4
                        local.get 2
                        local.get 0
                        i32.add
                        local.tee 0
                        local.get 0
                        i32.load offset=4
                        i32.const 1
                        i32.or
                        i32.store offset=4
                        br 1 (;@9;)
                      end
                      i32.const 0
                      local.get 7
                      i32.store offset=1059544
                      i32.const 0
                      local.get 2
                      local.get 3
                      i32.add
                      local.tee 6
                      i32.store offset=1059552
                      local.get 6
                      local.get 7
                      i32.const 1
                      i32.or
                      i32.store offset=4
                      local.get 2
                      local.get 0
                      i32.add
                      local.get 7
                      i32.store
                      local.get 2
                      local.get 3
                      i32.const 3
                      i32.or
                      i32.store offset=4
                    end
                    local.get 2
                    i32.const 8
                    i32.add
                    local.set 0
                    br 7 (;@1;)
                  end
                  local.get 0
                  local.get 8
                  local.get 9
                  i32.add
                  i32.store offset=4
                  i32.const 0
                  i32.const 0
                  i32.load offset=1059556
                  local.tee 0
                  i32.const 15
                  i32.add
                  i32.const -8
                  i32.and
                  local.tee 2
                  i32.const -8
                  i32.add
                  local.tee 7
                  i32.store offset=1059556
                  i32.const 0
                  local.get 0
                  local.get 2
                  i32.sub
                  i32.const 0
                  i32.load offset=1059548
                  local.get 9
                  i32.add
                  local.tee 2
                  i32.add
                  i32.const 8
                  i32.add
                  local.tee 6
                  i32.store offset=1059548
                  local.get 7
                  local.get 6
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  local.get 0
                  local.get 2
                  i32.add
                  i32.const 40
                  i32.store offset=4
                  i32.const 0
                  i32.const 2097152
                  i32.store offset=1059568
                  br 3 (;@4;)
                end
                i32.const 0
                local.get 0
                i32.store offset=1059556
                i32.const 0
                i32.const 0
                i32.load offset=1059548
                local.get 3
                i32.add
                local.tee 3
                i32.store offset=1059548
                local.get 0
                local.get 3
                i32.const 1
                i32.or
                i32.store offset=4
                br 1 (;@5;)
              end
              i32.const 0
              local.get 0
              i32.store offset=1059552
              i32.const 0
              i32.const 0
              i32.load offset=1059544
              local.get 3
              i32.add
              local.tee 3
              i32.store offset=1059544
              local.get 0
              local.get 3
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 0
              local.get 3
              i32.add
              local.get 3
              i32.store
            end
            local.get 7
            i32.const 8
            i32.add
            local.set 0
            br 3 (;@1;)
          end
          i32.const 0
          local.set 0
          i32.const 0
          i32.load offset=1059548
          local.tee 2
          local.get 3
          i32.le_u
          br_if 2 (;@1;)
          i32.const 0
          local.get 2
          local.get 3
          i32.sub
          local.tee 2
          i32.store offset=1059548
          i32.const 0
          i32.const 0
          i32.load offset=1059556
          local.tee 0
          local.get 3
          i32.add
          local.tee 7
          i32.store offset=1059556
          local.get 7
          local.get 2
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 0
          local.get 3
          i32.const 3
          i32.or
          i32.store offset=4
          local.get 0
          i32.const 8
          i32.add
          local.set 0
          br 2 (;@1;)
        end
        local.get 0
        local.get 5
        i32.store offset=24
        block  ;; label = @3
          local.get 7
          i32.load offset=16
          local.tee 6
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          local.get 6
          i32.store offset=16
          local.get 6
          local.get 0
          i32.store offset=24
        end
        local.get 7
        i32.load offset=20
        local.tee 6
        i32.eqz
        br_if 0 (;@2;)
        local.get 0
        local.get 6
        i32.store offset=20
        local.get 6
        local.get 0
        i32.store offset=24
      end
      block  ;; label = @2
        block  ;; label = @3
          local.get 2
          i32.const 16
          i32.lt_u
          br_if 0 (;@3;)
          local.get 7
          local.get 3
          i32.const 3
          i32.or
          i32.store offset=4
          local.get 7
          local.get 3
          i32.add
          local.tee 0
          local.get 2
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 0
          local.get 2
          i32.add
          local.get 2
          i32.store
          block  ;; label = @4
            local.get 2
            i32.const 256
            i32.lt_u
            br_if 0 (;@4;)
            local.get 0
            local.get 2
            call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$18insert_large_chunk17h368e2b1ffaf0fd0dE
            br 2 (;@2;)
          end
          local.get 2
          i32.const 248
          i32.and
          i32.const 1059272
          i32.add
          local.set 3
          block  ;; label = @4
            block  ;; label = @5
              i32.const 0
              i32.load offset=1059536
              local.tee 6
              i32.const 1
              local.get 2
              i32.const 3
              i32.shr_u
              i32.shl
              local.tee 2
              i32.and
              br_if 0 (;@5;)
              i32.const 0
              local.get 6
              local.get 2
              i32.or
              i32.store offset=1059536
              local.get 3
              local.set 2
              br 1 (;@4;)
            end
            local.get 3
            i32.load offset=8
            local.set 2
          end
          local.get 3
          local.get 0
          i32.store offset=8
          local.get 2
          local.get 0
          i32.store offset=12
          local.get 0
          local.get 3
          i32.store offset=12
          local.get 0
          local.get 2
          i32.store offset=8
          br 1 (;@2;)
        end
        local.get 7
        local.get 2
        local.get 3
        i32.add
        local.tee 0
        i32.const 3
        i32.or
        i32.store offset=4
        local.get 7
        local.get 0
        i32.add
        local.tee 0
        local.get 0
        i32.load offset=4
        i32.const 1
        i32.or
        i32.store offset=4
      end
      local.get 7
      i32.const 8
      i32.add
      local.set 0
    end
    local.get 1
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$8memalign17hf69f393c6806280dE (type 4) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32)
    i32.const 0
    local.set 2
    block  ;; label = @1
      i32.const -65587
      local.get 0
      i32.const 16
      local.get 0
      i32.const 16
      i32.gt_u
      select
      local.tee 0
      i32.sub
      local.get 1
      i32.le_u
      br_if 0 (;@1;)
      local.get 0
      i32.const 16
      local.get 1
      i32.const 11
      i32.add
      i32.const -8
      i32.and
      local.get 1
      i32.const 11
      i32.lt_u
      select
      local.tee 3
      i32.add
      i32.const 12
      i32.add
      call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$6malloc17hf8a67dcfc015198cE
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.const -8
      i32.add
      local.set 2
      block  ;; label = @2
        block  ;; label = @3
          local.get 0
          i32.const -1
          i32.add
          local.tee 4
          local.get 1
          i32.and
          br_if 0 (;@3;)
          local.get 2
          local.set 0
          br 1 (;@2;)
        end
        local.get 1
        i32.const -4
        i32.add
        local.tee 5
        i32.load
        local.tee 6
        i32.const -8
        i32.and
        local.get 4
        local.get 1
        i32.add
        i32.const 0
        local.get 0
        i32.sub
        i32.and
        i32.const -8
        i32.add
        local.tee 1
        i32.const 0
        local.get 0
        local.get 1
        local.get 2
        i32.sub
        i32.const 16
        i32.gt_u
        select
        i32.add
        local.tee 0
        local.get 2
        i32.sub
        local.tee 1
        i32.sub
        local.set 4
        block  ;; label = @3
          local.get 6
          i32.const 3
          i32.and
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          local.get 4
          local.get 0
          i32.load offset=4
          i32.const 1
          i32.and
          i32.or
          i32.const 2
          i32.or
          i32.store offset=4
          local.get 0
          local.get 4
          i32.add
          local.tee 4
          local.get 4
          i32.load offset=4
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 5
          local.get 1
          local.get 5
          i32.load
          i32.const 1
          i32.and
          i32.or
          i32.const 2
          i32.or
          i32.store
          local.get 2
          local.get 1
          i32.add
          local.tee 4
          local.get 4
          i32.load offset=4
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 2
          local.get 1
          call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$13dispose_chunk17h0b59c7a467076600E
          br 1 (;@2;)
        end
        local.get 2
        i32.load
        local.set 2
        local.get 0
        local.get 4
        i32.store offset=4
        local.get 0
        local.get 2
        local.get 1
        i32.add
        i32.store
      end
      block  ;; label = @2
        local.get 0
        i32.load offset=4
        local.tee 1
        i32.const 3
        i32.and
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.const -8
        i32.and
        local.tee 2
        local.get 3
        i32.const 16
        i32.add
        i32.le_u
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        local.get 1
        i32.const 1
        i32.and
        i32.or
        i32.const 2
        i32.or
        i32.store offset=4
        local.get 0
        local.get 3
        i32.add
        local.tee 1
        local.get 2
        local.get 3
        i32.sub
        local.tee 3
        i32.const 3
        i32.or
        i32.store offset=4
        local.get 0
        local.get 2
        i32.add
        local.tee 2
        local.get 2
        i32.load offset=4
        i32.const 1
        i32.or
        i32.store offset=4
        local.get 1
        local.get 3
        call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$13dispose_chunk17h0b59c7a467076600E
      end
      local.get 0
      i32.const 8
      i32.add
      local.set 2
    end
    local.get 2)
  (func $_ZN3std3sys4sync4once10no_threads4Once4call17he939191108bb8a0cE (type 0) (param i32)
    (local i32 i32 i64 i64 i64 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  i32.const 0
                  i32.load8_u offset=1059024
                  br_table 0 (;@7;) 2 (;@5;) 6 (;@1;) 1 (;@6;) 0 (;@7;)
                end
                i32.const 0
                i32.const 2
                i32.store8 offset=1059024
                local.get 0
                i32.load8_u
                local.set 2
                local.get 0
                i32.const 0
                i32.store8
                local.get 2
                i32.eqz
                br_if 2 (;@4;)
                local.get 1
                i32.const 0
                i32.store8 offset=8
                block  ;; label = @7
                  block  ;; label = @8
                    i32.const 0
                    i32.load8_u offset=1059080
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 1
                    i32.const 8
                    i32.add
                    call $_ZN3std4sync9once_lock17OnceLock$LT$T$GT$10initialize17hc6e8110d3d222757E
                    local.get 1
                    i32.load8_u offset=8
                    i32.const 1
                    i32.and
                    br_if 1 (;@7;)
                  end
                  block  ;; label = @8
                    i32.const 0
                    i64.load offset=1059112
                    local.tee 3
                    i64.const 0
                    i64.ne
                    br_if 0 (;@8;)
                    i32.const 0
                    i64.load offset=1059120
                    local.set 4
                    loop  ;; label = @9
                      local.get 4
                      i64.const -1
                      i64.eq
                      br_if 6 (;@3;)
                      i32.const 0
                      local.get 4
                      i64.const 1
                      i64.add
                      local.tee 3
                      i32.const 0
                      i64.load offset=1059120
                      local.tee 5
                      local.get 5
                      local.get 4
                      i64.eq
                      local.tee 0
                      select
                      i64.store offset=1059120
                      local.get 5
                      local.set 4
                      local.get 0
                      i32.eqz
                      br_if 0 (;@9;)
                    end
                    i32.const 0
                    local.get 3
                    i64.store offset=1059112
                  end
                  block  ;; label = @8
                    block  ;; label = @9
                      local.get 3
                      i32.const 0
                      i64.load offset=1059040
                      i64.eq
                      br_if 0 (;@9;)
                      i32.const 0
                      i32.load8_u offset=1059052
                      local.set 2
                      i32.const 1
                      local.set 0
                      i32.const 0
                      i32.const 1
                      i32.store8 offset=1059052
                      local.get 2
                      br_if 2 (;@7;)
                      i32.const 0
                      local.get 3
                      i64.store offset=1059040
                      br 1 (;@8;)
                    end
                    i32.const 0
                    i32.load offset=1059048
                    local.tee 0
                    i32.const -1
                    i32.eq
                    br_if 1 (;@7;)
                    local.get 0
                    i32.const 1
                    i32.add
                    local.set 0
                  end
                  i32.const 0
                  local.set 2
                  i32.const 0
                  local.get 0
                  i32.store offset=1059048
                  i32.const 0
                  i32.load offset=1059056
                  br_if 5 (;@2;)
                  i32.const 0
                  i32.const -1
                  i32.store offset=1059056
                  block  ;; label = @8
                    i32.const 0
                    i32.load offset=1059060
                    local.tee 6
                    i32.eqz
                    br_if 0 (;@8;)
                    i32.const 0
                    i32.load offset=1059064
                    local.get 6
                    i32.const 1
                    call $__rust_dealloc
                    i32.const 0
                    i32.load offset=1059056
                    i32.const 1
                    i32.add
                    local.set 2
                    i32.const 0
                    i32.load offset=1059048
                    local.set 0
                  end
                  i32.const 0
                  i64.const 4294967296
                  i64.store offset=1059060 align=4
                  i32.const 0
                  local.get 2
                  i32.store offset=1059056
                  i32.const 0
                  local.get 0
                  i32.const -1
                  i32.add
                  local.tee 0
                  i32.store offset=1059048
                  i32.const 0
                  i32.const 0
                  i32.store8 offset=1059072
                  i32.const 0
                  i32.const 0
                  i32.store offset=1059068
                  local.get 0
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.const 0
                  i64.store offset=1059040
                  i32.const 0
                  i32.const 0
                  i32.store8 offset=1059052
                end
                i32.const 0
                i32.const 3
                i32.store8 offset=1059024
              end
              local.get 1
              i32.const 32
              i32.add
              global.set $__stack_pointer
              return
            end
            local.get 1
            i32.const 0
            i32.store offset=24
            local.get 1
            i32.const 1
            i32.store offset=12
            local.get 1
            i32.const 1054912
            i32.store offset=8
            local.get 1
            i64.const 4
            i64.store offset=16 align=4
            local.get 1
            i32.const 8
            i32.add
            i32.const 1053308
            call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
            unreachable
          end
          i32.const 1053944
          call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
          unreachable
        end
        call $_ZN3std6thread8ThreadId3new9exhausted17hedd9ea9cc923223aE
        unreachable
      end
      i32.const 1053908
      call $_ZN4core4cell22panic_already_borrowed17h0fba8746ae6f569aE
      unreachable
    end
    local.get 1
    i32.const 0
    i32.store offset=24
    local.get 1
    i32.const 1
    i32.store offset=12
    local.get 1
    i32.const 1054976
    i32.store offset=8
    local.get 1
    i64.const 4
    i64.store offset=16 align=4
    local.get 1
    i32.const 8
    i32.add
    i32.const 1053308
    call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
    unreachable)
  (func $_ZN3std2rt19lang_start_internal17hdc6030aca1dd7348E (type 13) (param i32 i32 i32 i32 i32) (result i32)
    (local i32 i64 i64 i64 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 5
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          i32.const 0
          i64.load offset=1059112
          local.tee 6
          i64.const 0
          i64.ne
          br_if 0 (;@3;)
          i32.const 0
          i64.load offset=1059120
          local.set 7
          loop  ;; label = @4
            local.get 7
            i64.const -1
            i64.eq
            br_if 2 (;@2;)
            i32.const 0
            local.get 7
            i64.const 1
            i64.add
            local.tee 6
            i32.const 0
            i64.load offset=1059120
            local.tee 8
            local.get 8
            local.get 7
            i64.eq
            local.tee 9
            select
            i64.store offset=1059120
            local.get 8
            local.set 7
            local.get 9
            i32.eqz
            br_if 0 (;@4;)
          end
          i32.const 0
          local.get 6
          i64.store offset=1059112
        end
        i32.const 0
        local.get 6
        i64.store offset=1059032
        i32.const 0
        i32.load offset=1059592
        br_if 1 (;@1;)
        i32.const 0
        i32.const 1059032
        i32.store offset=1059592
        local.get 0
        local.get 1
        i32.load offset=20
        call_indirect (type 5)
        local.set 9
        block  ;; label = @3
          i32.const 0
          i32.load8_u offset=1059024
          i32.const 3
          i32.eq
          br_if 0 (;@3;)
          local.get 5
          i32.const 1
          i32.store8 offset=15
          local.get 5
          i32.const 15
          i32.add
          call $_ZN3std3sys4sync4once10no_threads4Once4call17he939191108bb8a0cE
        end
        local.get 5
        i32.const 16
        i32.add
        global.set $__stack_pointer
        local.get 9
        return
      end
      call $_ZN3std6thread8ThreadId3new9exhausted17hedd9ea9cc923223aE
    end
    unreachable)
  (func $_ZN3std6thread8ThreadId3new9exhausted17hedd9ea9cc923223aE (type 1)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 0
    global.set $__stack_pointer
    local.get 0
    i32.const 0
    i32.store offset=24
    local.get 0
    i32.const 1
    i32.store offset=12
    local.get 0
    i32.const 1053592
    i32.store offset=8
    local.get 0
    i64.const 4
    i64.store offset=16 align=4
    local.get 0
    i32.const 8
    i32.add
    i32.const 1053600
    call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
    unreachable)
  (func $_ZN3std6thread6scoped9ScopeData29increment_num_running_threads17h1759ddd5d0975e85E (type 0) (param i32)
    (local i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 0
    local.get 0
    i32.load offset=8
    local.tee 2
    i32.const 1
    i32.add
    i32.store offset=8
    block  ;; label = @1
      local.get 2
      i32.const -1
      i32.gt_s
      br_if 0 (;@1;)
      local.get 0
      local.get 0
      i32.load offset=8
      i32.const -1
      i32.add
      i32.store offset=8
      local.get 1
      i32.const 0
      i32.store offset=24
      local.get 1
      i32.const 1
      i32.store offset=12
      local.get 1
      i32.const 1053364
      i32.store offset=8
      local.get 1
      i64.const 4
      i64.store offset=16 align=4
      local.get 1
      i32.const 8
      i32.add
      i32.const 1053396
      call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
      unreachable
    end
    local.get 1
    i32.const 32
    i32.add
    global.set $__stack_pointer)
  (func $_ZN3std6thread6scoped9ScopeData29decrement_num_running_threads17hde25e83d0184f65dE (type 2) (param i32 i32)
    block  ;; label = @1
      local.get 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.const 1
      i32.store8 offset=12
    end
    local.get 0
    local.get 0
    i32.load offset=8
    i32.const -1
    i32.add
    i32.store offset=8)
  (func $_ZN3std6thread7current11set_current17h37b1d56a4498a5f9E (type 3) (param i32 i32 i32)
    (local i64 i64)
    block  ;; label = @1
      i32.const 0
      i32.load offset=1059592
      br_if 0 (;@1;)
      local.get 2
      i32.const 8
      i32.const 0
      local.get 1
      i32.const 1
      i32.and
      select
      i32.add
      i64.load
      local.set 3
      block  ;; label = @2
        block  ;; label = @3
          i32.const 0
          i64.load offset=1059112
          local.tee 4
          i64.const 0
          i64.ne
          br_if 0 (;@3;)
          i32.const 0
          local.get 3
          i64.store offset=1059112
          br 1 (;@2;)
        end
        local.get 4
        local.get 3
        i64.ne
        br_if 1 (;@1;)
      end
      i32.const 0
      local.get 2
      i32.const 8
      i32.const 0
      local.get 1
      i32.const 1
      i32.and
      select
      i32.add
      i32.store offset=1059592
      i32.const 2
      local.set 1
    end
    local.get 0
    local.get 2
    i32.store offset=4
    local.get 0
    local.get 1
    i32.store)
  (func $_ZN76_$LT$std..thread..spawnhook..SpawnHooks$u20$as$u20$core..ops..drop..Drop$GT$4drop17h3c4432d513a90f9bE (type 0) (param i32)
    (local i32 i32 i32 i32)
    local.get 0
    i32.load
    local.set 1
    local.get 0
    i32.const 0
    i32.store
    block  ;; label = @1
      local.get 1
      i32.eqz
      br_if 0 (;@1;)
      loop  ;; label = @2
        local.get 1
        local.get 1
        i32.load
        local.tee 0
        i32.const -1
        i32.add
        i32.store
        local.get 0
        i32.const 1
        i32.ne
        br_if 1 (;@1;)
        local.get 1
        i32.load offset=16
        local.set 2
        local.get 1
        i32.load offset=12
        local.set 0
        local.get 1
        i32.load offset=8
        local.set 3
        block  ;; label = @3
          local.get 1
          i32.const -1
          i32.eq
          br_if 0 (;@3;)
          local.get 1
          local.get 1
          i32.load offset=4
          local.tee 4
          i32.const -1
          i32.add
          i32.store offset=4
          local.get 4
          i32.const 1
          i32.ne
          br_if 0 (;@3;)
          local.get 1
          i32.const 20
          i32.const 4
          call $__rust_dealloc
        end
        local.get 3
        i32.eqz
        br_if 1 (;@1;)
        block  ;; label = @3
          local.get 0
          i32.load
          local.tee 1
          i32.eqz
          br_if 0 (;@3;)
          local.get 3
          local.get 1
          call_indirect (type 0)
        end
        block  ;; label = @3
          local.get 0
          i32.load offset=4
          local.tee 1
          i32.eqz
          br_if 0 (;@3;)
          local.get 3
          local.get 1
          local.get 0
          i32.load offset=8
          call $__rust_dealloc
        end
        local.get 2
        local.set 1
        local.get 2
        br_if 0 (;@2;)
      end
    end)
  (func $_ZN3std6thread9spawnhook15run_spawn_hooks17h0555442308226df3E (type 2) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    i32.const 0
    i32.load offset=1059588
    local.set 3
    i32.const 0
    i32.const 0
    i32.store offset=1059588
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 3
            i32.eqz
            br_if 0 (;@4;)
            local.get 3
            local.get 3
            i32.load
            local.tee 4
            i32.const 1
            i32.add
            i32.store
            block  ;; label = @5
              local.get 4
              i32.const 0
              i32.lt_s
              br_if 0 (;@5;)
              i32.const 0
              i32.load offset=1059588
              local.set 4
              i32.const 0
              local.get 3
              i32.store offset=1059588
              local.get 2
              local.get 4
              i32.store offset=20
              local.get 2
              i32.const 20
              i32.add
              call $_ZN4core3ptr55drop_in_place$LT$std..thread..spawnhook..SpawnHooks$GT$17hdd938607e6022e9bE
              local.get 3
              i32.load offset=16
              local.set 4
              local.get 2
              i32.const 8
              i32.add
              local.get 3
              i32.load offset=8
              local.get 1
              local.get 3
              i32.const 12
              i32.add
              i32.load
              i32.load offset=20
              call_indirect (type 3)
              local.get 2
              i32.load offset=8
              local.tee 5
              i32.eqz
              br_if 2 (;@3;)
              local.get 2
              i32.load offset=12
              local.set 6
              i32.const 0
              i32.load8_u offset=1058985
              drop
              i32.const 4
              local.set 7
              i32.const 32
              i32.const 4
              call $__rust_alloc
              local.tee 8
              i32.eqz
              br_if 4 (;@1;)
              local.get 8
              local.get 6
              i32.store offset=4
              local.get 8
              local.get 5
              i32.store
              i32.const 1
              local.set 5
              local.get 2
              i32.const 1
              i32.store offset=28
              local.get 2
              local.get 8
              i32.store offset=24
              local.get 2
              i32.const 4
              i32.store offset=20
              local.get 4
              i32.eqz
              br_if 3 (;@2;)
              i32.const 12
              local.set 9
              i32.const 1
              local.set 5
              block  ;; label = @6
                loop  ;; label = @7
                  local.get 4
                  i32.load offset=16
                  local.set 6
                  local.get 2
                  local.get 4
                  i32.load offset=8
                  local.get 1
                  local.get 4
                  i32.const 12
                  i32.add
                  i32.load
                  i32.load offset=20
                  call_indirect (type 3)
                  local.get 2
                  i32.load offset=20
                  local.set 7
                  local.get 2
                  i32.load
                  local.tee 4
                  i32.eqz
                  br_if 1 (;@6;)
                  local.get 2
                  i32.load offset=4
                  local.set 10
                  block  ;; label = @8
                    local.get 5
                    local.get 7
                    i32.ne
                    br_if 0 (;@8;)
                    local.get 2
                    i32.const 20
                    i32.add
                    local.get 5
                    i32.const 2
                    i32.const 1
                    local.get 6
                    select
                    i32.const 4
                    i32.const 8
                    call $_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17h15fb7f1ec1c750fcE
                    local.get 2
                    i32.load offset=24
                    local.set 8
                  end
                  local.get 8
                  local.get 9
                  i32.add
                  local.tee 7
                  local.get 10
                  i32.store
                  local.get 7
                  i32.const -4
                  i32.add
                  local.get 4
                  i32.store
                  local.get 2
                  local.get 5
                  i32.const 1
                  i32.add
                  local.tee 5
                  i32.store offset=28
                  local.get 9
                  i32.const 8
                  i32.add
                  local.set 9
                  local.get 6
                  local.set 4
                  local.get 6
                  br_if 0 (;@7;)
                end
                local.get 2
                i32.load offset=20
                local.set 7
              end
              local.get 2
              i32.load offset=24
              local.set 8
              br 3 (;@2;)
            end
            unreachable
          end
          local.get 2
          i32.const 0
          i32.store offset=20
          local.get 2
          i32.const 20
          i32.add
          call $_ZN4core3ptr55drop_in_place$LT$std..thread..spawnhook..SpawnHooks$GT$17hdd938607e6022e9bE
        end
        i32.const 4
        local.set 8
        i32.const 0
        local.set 5
        i32.const 0
        local.set 7
      end
      local.get 0
      local.get 5
      i32.store offset=8
      local.get 0
      local.get 8
      i32.store offset=4
      local.get 0
      local.get 7
      i32.store
      local.get 0
      local.get 3
      i32.store offset=12
      local.get 2
      i32.const 32
      i32.add
      global.set $__stack_pointer
      return
    end
    i32.const 4
    i32.const 32
    i32.const 1053500
    call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
    unreachable)
  (func $_ZN3std6thread9spawnhook15ChildSpawnHooks3run17h36c48039c3942367E (type 0) (param i32)
    (local i32 i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    i32.const 0
    i32.load offset=1059588
    local.set 2
    i32.const 0
    local.get 0
    i32.load offset=12
    i32.store offset=1059588
    local.get 1
    local.get 2
    i32.store offset=12
    local.get 1
    i32.const 12
    i32.add
    call $_ZN4core3ptr55drop_in_place$LT$std..thread..spawnhook..SpawnHooks$GT$17hdd938607e6022e9bE
    local.get 0
    i32.load offset=4
    local.set 3
    local.get 0
    i32.load
    local.set 4
    block  ;; label = @1
      local.get 0
      i32.load offset=8
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      local.get 3
      local.get 0
      i32.const 3
      i32.shl
      i32.add
      local.set 5
      local.get 3
      local.set 0
      loop  ;; label = @2
        local.get 0
        i32.load
        local.tee 6
        local.get 0
        i32.load offset=4
        local.tee 2
        i32.load offset=12
        call_indirect (type 0)
        block  ;; label = @3
          local.get 2
          i32.load offset=4
          local.tee 7
          i32.eqz
          br_if 0 (;@3;)
          local.get 6
          local.get 7
          local.get 2
          i32.load offset=8
          call $__rust_dealloc
        end
        local.get 0
        i32.const 8
        i32.add
        local.tee 0
        local.get 5
        i32.ne
        br_if 0 (;@2;)
      end
    end
    block  ;; label = @1
      local.get 4
      i32.eqz
      br_if 0 (;@1;)
      local.get 3
      local.get 4
      i32.const 3
      i32.shl
      i32.const 4
      call $__rust_dealloc
    end
    local.get 1
    i32.const 16
    i32.add
    global.set $__stack_pointer)
  (func $_ZN3std6thread7Builder4name17h4203e57084d3f707E (type 3) (param i32 i32 i32)
    (local i32 i32)
    local.get 1
    i32.const 8
    i32.add
    local.set 3
    block  ;; label = @1
      local.get 1
      i32.load offset=8
      local.tee 4
      i32.const -2147483648
      i32.or
      i32.const -2147483648
      i32.eq
      br_if 0 (;@1;)
      local.get 1
      i32.load offset=12
      local.get 4
      i32.const 1
      call $__rust_dealloc
    end
    local.get 3
    local.get 2
    i64.load align=4
    i64.store align=4
    local.get 3
    i32.const 8
    i32.add
    local.get 2
    i32.const 8
    i32.add
    i32.load
    i32.store
    local.get 0
    local.get 1
    i64.load align=4
    i64.store align=4
    local.get 0
    i32.const 8
    i32.add
    local.get 3
    i64.load align=4
    i64.store align=4
    local.get 0
    i32.const 16
    i32.add
    local.get 1
    i32.const 16
    i32.add
    i64.load align=4
    i64.store align=4)
  (func $_ZN3std6thread8ThreadId3new17hb3378d65506352f2E (type 14) (result i64)
    (local i64 i64 i64 i32)
    i32.const 0
    i64.load offset=1059120
    local.set 0
    block  ;; label = @1
      loop  ;; label = @2
        local.get 0
        i64.const -1
        i64.eq
        br_if 1 (;@1;)
        i32.const 0
        local.get 0
        i64.const 1
        i64.add
        local.tee 1
        i32.const 0
        i64.load offset=1059120
        local.tee 2
        local.get 2
        local.get 0
        i64.eq
        local.tee 3
        select
        i64.store offset=1059120
        local.get 2
        local.set 0
        local.get 3
        i32.eqz
        br_if 0 (;@2;)
      end
      local.get 1
      return
    end
    call $_ZN3std6thread8ThreadId3new9exhausted17hedd9ea9cc923223aE
    unreachable)
  (func $_ZN3std6thread6Thread3new17hc7b0b42432d00df2E (type 15) (param i32 i64 i32)
    (local i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 2
    i32.load offset=4
    local.set 4
    local.get 2
    i32.load
    local.set 5
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 2
            i32.load offset=8
            local.tee 6
            i32.const 7
            i32.gt_u
            br_if 0 (;@4;)
            local.get 6
            i32.eqz
            br_if 2 (;@2;)
            block  ;; label = @5
              local.get 4
              i32.load8_u
              br_if 0 (;@5;)
              i32.const 0
              local.set 2
              br 2 (;@3;)
            end
            i32.const 1
            local.set 2
            local.get 6
            i32.const 1
            i32.eq
            br_if 2 (;@2;)
            local.get 4
            i32.load8_u offset=1
            i32.eqz
            br_if 1 (;@3;)
            i32.const 2
            local.set 2
            local.get 6
            i32.const 2
            i32.eq
            br_if 2 (;@2;)
            local.get 4
            i32.load8_u offset=2
            i32.eqz
            br_if 1 (;@3;)
            i32.const 3
            local.set 2
            local.get 6
            i32.const 3
            i32.eq
            br_if 2 (;@2;)
            local.get 4
            i32.load8_u offset=3
            i32.eqz
            br_if 1 (;@3;)
            i32.const 4
            local.set 2
            local.get 6
            i32.const 4
            i32.eq
            br_if 2 (;@2;)
            local.get 4
            i32.load8_u offset=4
            i32.eqz
            br_if 1 (;@3;)
            i32.const 5
            local.set 2
            local.get 6
            i32.const 5
            i32.eq
            br_if 2 (;@2;)
            local.get 4
            i32.load8_u offset=5
            i32.eqz
            br_if 1 (;@3;)
            i32.const 6
            local.set 2
            local.get 6
            i32.const 6
            i32.eq
            br_if 2 (;@2;)
            local.get 4
            i32.load8_u offset=6
            i32.eqz
            br_if 1 (;@3;)
            br 2 (;@2;)
          end
          local.get 3
          i32.const 24
          i32.add
          i32.const 0
          local.get 4
          local.get 6
          call $_ZN4core5slice6memchr14memchr_aligned17hc466838cdf21c242E
          local.get 3
          i32.load offset=24
          i32.eqz
          br_if 1 (;@2;)
          local.get 3
          i32.load offset=28
          local.set 2
        end
        local.get 5
        i32.const -2147483648
        i32.eq
        br_if 1 (;@1;)
        local.get 3
        local.get 2
        i32.store offset=44
        local.get 3
        local.get 6
        i32.store offset=40
        local.get 3
        local.get 4
        i32.store offset=36
        local.get 3
        local.get 5
        i32.store offset=32
        i32.const 1053632
        i32.const 47
        local.get 3
        i32.const 32
        i32.add
        i32.const 1053616
        i32.const 1053680
        call $_ZN4core6result13unwrap_failed17h89eac97f11bebdf4E
        unreachable
      end
      local.get 3
      local.get 6
      i32.store offset=40
      local.get 3
      local.get 4
      i32.store offset=36
      local.get 3
      local.get 5
      i32.store offset=32
      local.get 3
      i32.const 16
      i32.add
      local.get 3
      i32.const 32
      i32.add
      call $_ZN5alloc3ffi5c_str7CString19_from_vec_unchecked17hddc42bd4ebb2f1c3E
      local.get 3
      i32.load offset=20
      local.set 6
      local.get 3
      i32.load offset=16
      local.set 4
    end
    local.get 3
    i32.const 8
    i32.add
    i32.const 8
    i32.const 16
    call $_ZN5alloc4sync32arcinner_layout_for_value_layout17hea65de36431e6179E
    local.get 3
    i32.load offset=8
    local.set 5
    block  ;; label = @1
      block  ;; label = @2
        local.get 3
        i32.load offset=12
        local.tee 7
        br_if 0 (;@2;)
        local.get 5
        local.set 2
        br 1 (;@1;)
      end
      i32.const 0
      i32.load8_u offset=1058985
      drop
      local.get 7
      local.get 5
      call $__rust_alloc
      local.set 2
    end
    block  ;; label = @1
      local.get 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 6
      i32.store offset=20
      local.get 2
      local.get 4
      i32.store offset=16
      local.get 2
      i64.const 4294967297
      i64.store
      local.get 2
      local.get 1
      i64.store offset=8
      local.get 0
      local.get 2
      i32.store offset=4
      local.get 0
      i32.const 1
      i32.store
      local.get 3
      i32.const 48
      i32.add
      global.set $__stack_pointer
      return
    end
    local.get 5
    local.get 7
    call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
    unreachable)
  (func $_ZN3std6thread6Thread11new_unnamed17h5a535a66c682809cE (type 16) (param i32 i64)
    (local i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 8
    i32.add
    i32.const 8
    i32.const 16
    call $_ZN5alloc4sync32arcinner_layout_for_value_layout17hea65de36431e6179E
    local.get 2
    i32.load offset=8
    local.set 3
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        i32.load offset=12
        local.tee 4
        br_if 0 (;@2;)
        local.get 3
        local.set 5
        br 1 (;@1;)
      end
      i32.const 0
      i32.load8_u offset=1058985
      drop
      local.get 4
      local.get 3
      call $__rust_alloc
      local.set 5
    end
    block  ;; label = @1
      local.get 5
      br_if 0 (;@1;)
      local.get 3
      local.get 4
      call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
      unreachable
    end
    local.get 5
    i32.const 0
    i32.store offset=16
    local.get 5
    i64.const 4294967297
    i64.store
    local.get 5
    local.get 1
    i64.store offset=8
    local.get 0
    local.get 5
    i32.store offset=4
    local.get 0
    i32.const 1
    i32.store
    local.get 2
    i32.const 16
    i32.add
    global.set $__stack_pointer)
  (func $_ZN3std6thread6Thread5cname17h8efee6976efe2f09E (type 2) (param i32 i32)
    (local i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.load
        i32.const 1
        i32.eq
        br_if 0 (;@2;)
        i32.const 1053696
        local.set 1
        i32.const 5
        local.set 2
        br 1 (;@1;)
      end
      block  ;; label = @2
        local.get 1
        i32.load offset=4
        local.tee 2
        i32.load offset=16
        local.tee 1
        br_if 0 (;@2;)
        i32.const 0
        local.set 1
        br 1 (;@1;)
      end
      local.get 2
      i32.load offset=20
      local.set 2
    end
    local.get 0
    local.get 2
    i32.store offset=4
    local.get 0
    local.get 1
    i32.store)
  (func $_ZN58_$LT$std..io..error..Error$u20$as$u20$core..fmt..Debug$GT$3fmt17h596e914530403c3cE (type 4) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    call $_ZN3std2io5error82_$LT$impl$u20$core..fmt..Debug$u20$for$u20$std..io..error..repr_unpacked..Repr$GT$3fmt17ha621a9867020956eE)
  (func $_ZN3std2io5error82_$LT$impl$u20$core..fmt..Debug$u20$for$u20$std..io..error..repr_unpacked..Repr$GT$3fmt17ha621a9867020956eE (type 4) (param i32 i32) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                local.get 0
                i32.load8_u
                br_table 0 (;@6;) 1 (;@5;) 2 (;@4;) 3 (;@3;) 0 (;@6;)
              end
              local.get 2
              local.get 0
              i32.load offset=4
              i32.store offset=4
              local.get 2
              i32.const 8
              i32.add
              local.get 1
              i32.const 1053768
              i32.const 2
              call $_ZN4core3fmt9Formatter12debug_struct17he54794dbb5a1813cE
              local.get 2
              i32.const 8
              i32.add
              i32.const 1053788
              i32.const 4
              local.get 2
              i32.const 4
              i32.add
              i32.const 1053772
              call $_ZN4core3fmt8builders11DebugStruct5field17h30cdc1b516ecb392E
              local.set 0
              local.get 2
              i32.const 41
              i32.store8 offset=19
              local.get 0
              i32.const 1053808
              i32.const 4
              local.get 2
              i32.const 19
              i32.add
              i32.const 1053792
              call $_ZN4core3fmt8builders11DebugStruct5field17h30cdc1b516ecb392E
              local.set 1
              i32.const 0
              i32.load8_u offset=1058985
              drop
              i32.const 20
              i32.const 1
              call $__rust_alloc
              local.tee 0
              i32.eqz
              br_if 4 (;@1;)
              local.get 0
              i32.const 16
              i32.add
              i32.const 0
              i32.load offset=1054772 align=1
              i32.store align=1
              local.get 0
              i32.const 8
              i32.add
              i32.const 0
              i64.load offset=1054764 align=1
              i64.store align=1
              local.get 0
              i32.const 0
              i64.load offset=1054756 align=1
              i64.store align=1
              local.get 2
              i32.const 20
              i32.store offset=28
              local.get 2
              local.get 0
              i32.store offset=24
              local.get 2
              i32.const 20
              i32.store offset=20
              local.get 1
              i32.const 1053828
              i32.const 7
              local.get 2
              i32.const 20
              i32.add
              i32.const 1053812
              call $_ZN4core3fmt8builders11DebugStruct5field17h30cdc1b516ecb392E
              call $_ZN4core3fmt8builders11DebugStruct6finish17h02d6fd6087b36dfdE
              local.set 0
              local.get 2
              i32.load offset=20
              local.tee 1
              i32.eqz
              br_if 3 (;@2;)
              local.get 2
              i32.load offset=24
              local.get 1
              i32.const 1
              call $__rust_dealloc
              br 3 (;@2;)
            end
            local.get 2
            local.get 0
            i32.load8_u offset=1
            i32.store8 offset=8
            local.get 2
            i32.const 20
            i32.add
            local.get 1
            i32.const 1053835
            i32.const 4
            call $_ZN4core3fmt9Formatter11debug_tuple17h8e8348afcbac700aE
            local.get 2
            i32.const 20
            i32.add
            local.get 2
            i32.const 8
            i32.add
            i32.const 1053792
            call $_ZN4core3fmt8builders10DebugTuple5field17h16ef0971d382d024E
            call $_ZN4core3fmt8builders10DebugTuple6finish17he75364125644b4c4E
            local.set 0
            br 2 (;@2;)
          end
          local.get 0
          i32.load offset=4
          local.set 0
          local.get 2
          i32.const 20
          i32.add
          local.get 1
          i32.const 1053839
          i32.const 5
          call $_ZN4core3fmt9Formatter12debug_struct17he54794dbb5a1813cE
          local.get 2
          i32.const 20
          i32.add
          i32.const 1053808
          i32.const 4
          local.get 0
          i32.const 8
          i32.add
          i32.const 1053792
          call $_ZN4core3fmt8builders11DebugStruct5field17h30cdc1b516ecb392E
          i32.const 1053828
          i32.const 7
          local.get 0
          i32.const 1053844
          call $_ZN4core3fmt8builders11DebugStruct5field17h30cdc1b516ecb392E
          call $_ZN4core3fmt8builders11DebugStruct6finish17h02d6fd6087b36dfdE
          local.set 0
          br 1 (;@2;)
        end
        local.get 2
        local.get 0
        i32.load offset=4
        local.tee 0
        i32.store offset=20
        local.get 1
        i32.const 1053876
        i32.const 6
        i32.const 1053808
        i32.const 4
        local.get 0
        i32.const 8
        i32.add
        i32.const 1053792
        i32.const 1053882
        i32.const 5
        local.get 2
        i32.const 20
        i32.add
        i32.const 1053860
        call $_ZN4core3fmt9Formatter26debug_struct_field2_finish17ha199c8f1cad09d1bE
        local.set 0
      end
      local.get 2
      i32.const 32
      i32.add
      global.set $__stack_pointer
      local.get 0
      return
    end
    i32.const 1
    i32.const 20
    i32.const 1052860
    call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
    unreachable)
  (func $_ZN3std5panic13resume_unwind17h2dd9bc0fe68fc10cE (type 2) (param i32 i32)
    local.get 0
    local.get 1
    call $_ZN3std9panicking23rust_panic_without_hook17ha2580b63e5b77ddfE
    unreachable)
  (func $_ZN3std9panicking23rust_panic_without_hook17ha2580b63e5b77ddfE (type 2) (param i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    i32.const 0
    call $_ZN3std9panicking11panic_count8increase17he00fd07a22feffa3E
    drop
    local.get 2
    local.get 1
    i32.store offset=12
    local.get 2
    local.get 0
    i32.store offset=8
    local.get 2
    i32.const 8
    i32.add
    i32.const 1054160
    call $rust_panic
    unreachable)
  (func $_ZN3std4sync9once_lock17OnceLock$LT$T$GT$10initialize17hc6e8110d3d222757E (type 0) (param i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          i32.const 0
          i32.load8_u offset=1059080
          br_table 1 (;@2;) 1 (;@2;) 0 (;@3;) 2 (;@1;) 1 (;@2;)
        end
        local.get 1
        i32.const 0
        i32.store offset=24
        local.get 1
        i32.const 1
        i32.store offset=12
        local.get 1
        i32.const 1054976
        i32.store offset=8
        local.get 1
        i64.const 4
        i64.store offset=16 align=4
        local.get 1
        i32.const 8
        i32.add
        i32.const 1053960
        call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
        unreachable
      end
      i32.const 0
      i32.const 3
      i32.store8 offset=1059080
      i32.const 0
      i64.const 1
      i64.store offset=1059064
      i32.const 0
      i64.const 0
      i64.store offset=1059056
      i32.const 0
      i64.const 0
      i64.store offset=1059040
      local.get 0
      i32.const 1
      i32.store8
      i32.const 0
      i32.const 0
      i32.store8 offset=1059072
      i32.const 0
      i32.const 0
      i32.store8 offset=1059052
      i32.const 0
      i32.const 0
      i32.store offset=1059048
    end
    local.get 1
    i32.const 32
    i32.add
    global.set $__stack_pointer)
  (func $_ZN3std3sys9backtrace26__rust_end_short_backtrace17h68c8fb62cb5b0572E (type 0) (param i32)
    local.get 0
    call $_ZN3std9panicking19begin_panic_handler28_$u7b$$u7b$closure$u7d$$u7d$17hc2bc40873e4ac2aaE
    unreachable)
  (func $_ZN3std9panicking19begin_panic_handler28_$u7b$$u7b$closure$u7d$$u7d$17hc2bc40873e4ac2aaE (type 0) (param i32)
    (local i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 0
    i32.load
    local.tee 2
    i32.load offset=12
    local.set 3
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 2
            i32.load offset=4
            br_table 0 (;@4;) 1 (;@3;) 2 (;@2;)
          end
          local.get 3
          br_if 1 (;@2;)
          i32.const 1
          local.set 2
          i32.const 0
          local.set 3
          br 2 (;@1;)
        end
        local.get 3
        br_if 0 (;@2;)
        local.get 2
        i32.load
        local.tee 2
        i32.load offset=4
        local.set 3
        local.get 2
        i32.load
        local.set 2
        br 1 (;@1;)
      end
      local.get 1
      i32.const -2147483648
      i32.store
      local.get 1
      local.get 0
      i32.store offset=12
      local.get 1
      i32.const 1054120
      local.get 0
      i32.load offset=4
      local.get 0
      i32.load offset=8
      local.tee 0
      i32.load8_u offset=8
      local.get 0
      i32.load8_u offset=9
      call $_ZN3std9panicking20rust_panic_with_hook17hcc2503a68438883eE
      unreachable
    end
    local.get 1
    local.get 3
    i32.store offset=4
    local.get 1
    local.get 2
    i32.store
    local.get 1
    i32.const 1054092
    local.get 0
    i32.load offset=4
    local.get 0
    i32.load offset=8
    local.tee 0
    i32.load8_u offset=8
    local.get 0
    i32.load8_u offset=9
    call $_ZN3std9panicking20rust_panic_with_hook17hcc2503a68438883eE
    unreachable)
  (func $_ZN3std5alloc24default_alloc_error_hook17hb6719f23c72b7373E (type 2) (param i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      i32.const 0
      i32.load8_u offset=1058984
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      i32.const 2
      i32.store offset=12
      local.get 2
      i32.const 1054012
      i32.store offset=8
      local.get 2
      i64.const 1
      i64.store offset=20 align=4
      local.get 2
      local.get 1
      i32.store offset=44
      local.get 2
      i32.const 22
      i64.extend_i32_u
      i64.const 32
      i64.shl
      local.get 2
      i32.const 44
      i32.add
      i64.extend_i32_u
      i64.or
      i64.store offset=32
      local.get 2
      local.get 2
      i32.const 32
      i32.add
      i32.store offset=16
      local.get 2
      i32.const 8
      i32.add
      i32.const 1054044
      call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
      unreachable
    end
    local.get 2
    i32.const 48
    i32.add
    global.set $__stack_pointer)
  (func $__rdl_alloc (type 4) (param i32 i32) (result i32)
    block  ;; label = @1
      local.get 1
      i32.const 9
      i32.lt_u
      br_if 0 (;@1;)
      local.get 1
      local.get 0
      call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$8memalign17hf69f393c6806280dE
      return
    end
    local.get 0
    call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$6malloc17hf8a67dcfc015198cE)
  (func $__rdl_dealloc (type 3) (param i32 i32 i32)
    (local i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.const -4
        i32.add
        i32.load
        local.tee 3
        i32.const -8
        i32.and
        local.tee 4
        i32.const 4
        i32.const 8
        local.get 3
        i32.const 3
        i32.and
        local.tee 3
        select
        local.get 1
        i32.add
        i32.lt_u
        br_if 0 (;@2;)
        block  ;; label = @3
          local.get 3
          i32.eqz
          br_if 0 (;@3;)
          local.get 4
          local.get 1
          i32.const 39
          i32.add
          i32.gt_u
          br_if 2 (;@1;)
        end
        local.get 0
        call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$4free17he5d00a6c001acd28E
        return
      end
      i32.const 1053165
      i32.const 46
      i32.const 1053212
      call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
      unreachable
    end
    i32.const 1053228
    i32.const 46
    i32.const 1053276
    call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
    unreachable)
  (func $__rdl_realloc (type 10) (param i32 i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 0
              i32.const -4
              i32.add
              local.tee 4
              i32.load
              local.tee 5
              i32.const -8
              i32.and
              local.tee 6
              i32.const 4
              i32.const 8
              local.get 5
              i32.const 3
              i32.and
              local.tee 7
              select
              local.get 1
              i32.add
              i32.lt_u
              br_if 0 (;@5;)
              local.get 1
              i32.const 39
              i32.add
              local.set 8
              block  ;; label = @6
                local.get 7
                i32.eqz
                br_if 0 (;@6;)
                local.get 6
                local.get 8
                i32.gt_u
                br_if 2 (;@4;)
              end
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 2
                    i32.const 9
                    i32.lt_u
                    br_if 0 (;@8;)
                    local.get 2
                    local.get 3
                    call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$8memalign17hf69f393c6806280dE
                    local.tee 2
                    br_if 1 (;@7;)
                    i32.const 0
                    return
                  end
                  i32.const 0
                  local.set 2
                  local.get 3
                  i32.const -65588
                  i32.gt_u
                  br_if 1 (;@6;)
                  i32.const 16
                  local.get 3
                  i32.const 11
                  i32.add
                  i32.const -8
                  i32.and
                  local.get 3
                  i32.const 11
                  i32.lt_u
                  select
                  local.set 1
                  block  ;; label = @8
                    block  ;; label = @9
                      local.get 7
                      br_if 0 (;@9;)
                      local.get 1
                      i32.const 256
                      i32.lt_u
                      br_if 1 (;@8;)
                      local.get 6
                      local.get 1
                      i32.const 4
                      i32.or
                      i32.lt_u
                      br_if 1 (;@8;)
                      local.get 6
                      local.get 1
                      i32.sub
                      i32.const 131073
                      i32.ge_u
                      br_if 1 (;@8;)
                      local.get 0
                      return
                    end
                    local.get 0
                    i32.const -8
                    i32.add
                    local.tee 8
                    local.get 6
                    i32.add
                    local.set 7
                    block  ;; label = @9
                      block  ;; label = @10
                        block  ;; label = @11
                          block  ;; label = @12
                            block  ;; label = @13
                              local.get 6
                              local.get 1
                              i32.ge_u
                              br_if 0 (;@13;)
                              local.get 7
                              i32.const 0
                              i32.load offset=1059556
                              i32.eq
                              br_if 4 (;@9;)
                              local.get 7
                              i32.const 0
                              i32.load offset=1059552
                              i32.eq
                              br_if 2 (;@11;)
                              local.get 7
                              i32.load offset=4
                              local.tee 5
                              i32.const 2
                              i32.and
                              br_if 5 (;@8;)
                              local.get 5
                              i32.const -8
                              i32.and
                              local.tee 9
                              local.get 6
                              i32.add
                              local.tee 5
                              local.get 1
                              i32.lt_u
                              br_if 5 (;@8;)
                              local.get 7
                              local.get 9
                              call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$12unlink_chunk17h92eb07d772b9ef18E
                              local.get 5
                              local.get 1
                              i32.sub
                              local.tee 3
                              i32.const 16
                              i32.lt_u
                              br_if 1 (;@12;)
                              local.get 4
                              local.get 1
                              local.get 4
                              i32.load
                              i32.const 1
                              i32.and
                              i32.or
                              i32.const 2
                              i32.or
                              i32.store
                              local.get 8
                              local.get 1
                              i32.add
                              local.tee 1
                              local.get 3
                              i32.const 3
                              i32.or
                              i32.store offset=4
                              local.get 8
                              local.get 5
                              i32.add
                              local.tee 2
                              local.get 2
                              i32.load offset=4
                              i32.const 1
                              i32.or
                              i32.store offset=4
                              local.get 1
                              local.get 3
                              call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$13dispose_chunk17h0b59c7a467076600E
                              local.get 0
                              return
                            end
                            local.get 6
                            local.get 1
                            i32.sub
                            local.tee 3
                            i32.const 15
                            i32.gt_u
                            br_if 2 (;@10;)
                            local.get 0
                            return
                          end
                          local.get 4
                          local.get 5
                          local.get 4
                          i32.load
                          i32.const 1
                          i32.and
                          i32.or
                          i32.const 2
                          i32.or
                          i32.store
                          local.get 8
                          local.get 5
                          i32.add
                          local.tee 1
                          local.get 1
                          i32.load offset=4
                          i32.const 1
                          i32.or
                          i32.store offset=4
                          local.get 0
                          return
                        end
                        i32.const 0
                        i32.load offset=1059544
                        local.get 6
                        i32.add
                        local.tee 7
                        local.get 1
                        i32.lt_u
                        br_if 2 (;@8;)
                        block  ;; label = @11
                          block  ;; label = @12
                            local.get 7
                            local.get 1
                            i32.sub
                            local.tee 3
                            i32.const 15
                            i32.gt_u
                            br_if 0 (;@12;)
                            local.get 4
                            local.get 5
                            i32.const 1
                            i32.and
                            local.get 7
                            i32.or
                            i32.const 2
                            i32.or
                            i32.store
                            local.get 8
                            local.get 7
                            i32.add
                            local.tee 1
                            local.get 1
                            i32.load offset=4
                            i32.const 1
                            i32.or
                            i32.store offset=4
                            i32.const 0
                            local.set 3
                            i32.const 0
                            local.set 1
                            br 1 (;@11;)
                          end
                          local.get 4
                          local.get 1
                          local.get 5
                          i32.const 1
                          i32.and
                          i32.or
                          i32.const 2
                          i32.or
                          i32.store
                          local.get 8
                          local.get 1
                          i32.add
                          local.tee 1
                          local.get 3
                          i32.const 1
                          i32.or
                          i32.store offset=4
                          local.get 8
                          local.get 7
                          i32.add
                          local.tee 2
                          local.get 3
                          i32.store
                          local.get 2
                          local.get 2
                          i32.load offset=4
                          i32.const -2
                          i32.and
                          i32.store offset=4
                        end
                        i32.const 0
                        local.get 1
                        i32.store offset=1059552
                        i32.const 0
                        local.get 3
                        i32.store offset=1059544
                        local.get 0
                        return
                      end
                      local.get 4
                      local.get 1
                      local.get 5
                      i32.const 1
                      i32.and
                      i32.or
                      i32.const 2
                      i32.or
                      i32.store
                      local.get 8
                      local.get 1
                      i32.add
                      local.tee 1
                      local.get 3
                      i32.const 3
                      i32.or
                      i32.store offset=4
                      local.get 7
                      local.get 7
                      i32.load offset=4
                      i32.const 1
                      i32.or
                      i32.store offset=4
                      local.get 1
                      local.get 3
                      call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$13dispose_chunk17h0b59c7a467076600E
                      local.get 0
                      return
                    end
                    i32.const 0
                    i32.load offset=1059548
                    local.get 6
                    i32.add
                    local.tee 7
                    local.get 1
                    i32.gt_u
                    br_if 7 (;@1;)
                  end
                  local.get 3
                  call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$6malloc17hf8a67dcfc015198cE
                  local.tee 1
                  i32.eqz
                  br_if 1 (;@6;)
                  local.get 1
                  local.get 0
                  i32.const -4
                  i32.const -8
                  local.get 4
                  i32.load
                  local.tee 2
                  i32.const 3
                  i32.and
                  select
                  local.get 2
                  i32.const -8
                  i32.and
                  i32.add
                  local.tee 2
                  local.get 3
                  local.get 2
                  local.get 3
                  i32.lt_u
                  select
                  call $memcpy
                  local.set 1
                  local.get 0
                  call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$4free17he5d00a6c001acd28E
                  local.get 1
                  return
                end
                local.get 2
                local.get 0
                local.get 1
                local.get 3
                local.get 1
                local.get 3
                i32.lt_u
                select
                call $memcpy
                drop
                local.get 4
                i32.load
                local.tee 3
                i32.const -8
                i32.and
                local.tee 7
                i32.const 4
                i32.const 8
                local.get 3
                i32.const 3
                i32.and
                local.tee 3
                select
                local.get 1
                i32.add
                i32.lt_u
                br_if 3 (;@3;)
                block  ;; label = @7
                  local.get 3
                  i32.eqz
                  br_if 0 (;@7;)
                  local.get 7
                  local.get 8
                  i32.gt_u
                  br_if 5 (;@2;)
                end
                local.get 0
                call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$4free17he5d00a6c001acd28E
              end
              local.get 2
              return
            end
            i32.const 1053165
            i32.const 46
            i32.const 1053212
            call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
            unreachable
          end
          i32.const 1053228
          i32.const 46
          i32.const 1053276
          call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
          unreachable
        end
        i32.const 1053165
        i32.const 46
        i32.const 1053212
        call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
        unreachable
      end
      i32.const 1053228
      i32.const 46
      i32.const 1053276
      call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
      unreachable
    end
    local.get 4
    local.get 1
    local.get 5
    i32.const 1
    i32.and
    i32.or
    i32.const 2
    i32.or
    i32.store
    local.get 8
    local.get 1
    i32.add
    local.tee 3
    local.get 7
    local.get 1
    i32.sub
    local.tee 1
    i32.const 1
    i32.or
    i32.store offset=4
    i32.const 0
    local.get 1
    i32.store offset=1059548
    i32.const 0
    local.get 3
    i32.store offset=1059556
    local.get 0)
  (func $__rdl_alloc_zeroed (type 4) (param i32 i32) (result i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.const 9
        i32.lt_u
        br_if 0 (;@2;)
        local.get 1
        local.get 0
        call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$8memalign17hf69f393c6806280dE
        local.set 1
        br 1 (;@1;)
      end
      local.get 0
      call $_ZN8dlmalloc8dlmalloc17Dlmalloc$LT$A$GT$6malloc17hf8a67dcfc015198cE
      local.set 1
    end
    block  ;; label = @1
      local.get 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.const -4
      i32.add
      i32.load8_u
      i32.const 3
      i32.and
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.const 0
      local.get 0
      call $memset
      drop
    end
    local.get 1)
  (func $_ZN3std9panicking11panic_count8increase17he00fd07a22feffa3E (type 5) (param i32) (result i32)
    (local i32 i32)
    i32.const 0
    local.set 1
    i32.const 0
    i32.const 0
    i32.load offset=1059104
    local.tee 2
    i32.const 1
    i32.add
    i32.store offset=1059104
    block  ;; label = @1
      local.get 2
      i32.const 0
      i32.lt_s
      br_if 0 (;@1;)
      i32.const 1
      local.set 1
      i32.const 0
      i32.load8_u offset=1059584
      br_if 0 (;@1;)
      i32.const 0
      local.get 0
      i32.store8 offset=1059584
      i32.const 0
      i32.const 0
      i32.load offset=1059580
      i32.const 1
      i32.add
      i32.store offset=1059580
      i32.const 2
      local.set 1
    end
    local.get 1)
  (func $rust_begin_unwind (type 0) (param i32)
    (local i32 i64)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 0
    i64.load align=4
    local.set 2
    local.get 1
    local.get 0
    i32.store offset=12
    local.get 1
    local.get 2
    i64.store offset=4 align=4
    local.get 1
    i32.const 4
    i32.add
    call $_ZN3std3sys9backtrace26__rust_end_short_backtrace17h68c8fb62cb5b0572E
    unreachable)
  (func $_ZN102_$LT$std..panicking..begin_panic_handler..FormatStringPayload$u20$as$u20$core..panic..PanicPayload$GT$8take_box17hd73901fc3ea6a706E (type 2) (param i32 i32)
    (local i32 i32 i32 i64)
    global.get $__stack_pointer
    i32.const 64
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      local.get 1
      i32.load
      i32.const -2147483648
      i32.ne
      br_if 0 (;@1;)
      local.get 1
      i32.load offset=12
      local.set 3
      local.get 2
      i32.const 28
      i32.add
      i32.const 8
      i32.add
      local.tee 4
      i32.const 0
      i32.store
      local.get 2
      i64.const 4294967296
      i64.store offset=28 align=4
      local.get 2
      i32.const 40
      i32.add
      i32.const 8
      i32.add
      local.get 3
      i32.load
      local.tee 3
      i32.const 8
      i32.add
      i64.load align=4
      i64.store
      local.get 2
      i32.const 40
      i32.add
      i32.const 16
      i32.add
      local.get 3
      i32.const 16
      i32.add
      i64.load align=4
      i64.store
      local.get 2
      local.get 3
      i64.load align=4
      i64.store offset=40
      local.get 2
      i32.const 28
      i32.add
      i32.const 1053100
      local.get 2
      i32.const 40
      i32.add
      call $_ZN4core3fmt5write17hcf5d300c090957a7E
      drop
      local.get 2
      i32.const 16
      i32.add
      i32.const 8
      i32.add
      local.get 4
      i32.load
      local.tee 3
      i32.store
      local.get 2
      local.get 2
      i64.load offset=28 align=4
      local.tee 5
      i64.store offset=16
      local.get 1
      i32.const 8
      i32.add
      local.get 3
      i32.store
      local.get 1
      local.get 5
      i64.store align=4
    end
    local.get 1
    i64.load align=4
    local.set 5
    local.get 1
    i64.const 4294967296
    i64.store align=4
    local.get 2
    i32.const 8
    i32.add
    local.tee 3
    local.get 1
    i32.const 8
    i32.add
    local.tee 1
    i32.load
    i32.store
    local.get 1
    i32.const 0
    i32.store
    i32.const 0
    i32.load8_u offset=1058985
    drop
    local.get 2
    local.get 5
    i64.store
    block  ;; label = @1
      i32.const 12
      i32.const 4
      call $__rust_alloc
      local.tee 1
      br_if 0 (;@1;)
      i32.const 4
      i32.const 12
      call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
      unreachable
    end
    local.get 1
    local.get 2
    i64.load
    i64.store align=4
    local.get 1
    i32.const 8
    i32.add
    local.get 3
    i32.load
    i32.store
    local.get 0
    i32.const 1054060
    i32.store offset=4
    local.get 0
    local.get 1
    i32.store
    local.get 2
    i32.const 64
    i32.add
    global.set $__stack_pointer)
  (func $_ZN102_$LT$std..panicking..begin_panic_handler..FormatStringPayload$u20$as$u20$core..panic..PanicPayload$GT$3get17h624ff36aab1cfa5dE (type 2) (param i32 i32)
    (local i32 i32 i32 i64)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      local.get 1
      i32.load
      i32.const -2147483648
      i32.ne
      br_if 0 (;@1;)
      local.get 1
      i32.load offset=12
      local.set 3
      local.get 2
      i32.const 12
      i32.add
      i32.const 8
      i32.add
      local.tee 4
      i32.const 0
      i32.store
      local.get 2
      i64.const 4294967296
      i64.store offset=12 align=4
      local.get 2
      i32.const 24
      i32.add
      i32.const 8
      i32.add
      local.get 3
      i32.load
      local.tee 3
      i32.const 8
      i32.add
      i64.load align=4
      i64.store
      local.get 2
      i32.const 24
      i32.add
      i32.const 16
      i32.add
      local.get 3
      i32.const 16
      i32.add
      i64.load align=4
      i64.store
      local.get 2
      local.get 3
      i64.load align=4
      i64.store offset=24
      local.get 2
      i32.const 12
      i32.add
      i32.const 1053100
      local.get 2
      i32.const 24
      i32.add
      call $_ZN4core3fmt5write17hcf5d300c090957a7E
      drop
      local.get 2
      i32.const 8
      i32.add
      local.get 4
      i32.load
      local.tee 3
      i32.store
      local.get 2
      local.get 2
      i64.load offset=12 align=4
      local.tee 5
      i64.store
      local.get 1
      i32.const 8
      i32.add
      local.get 3
      i32.store
      local.get 1
      local.get 5
      i64.store align=4
    end
    local.get 0
    i32.const 1054060
    i32.store offset=4
    local.get 0
    local.get 1
    i32.store
    local.get 2
    i32.const 48
    i32.add
    global.set $__stack_pointer)
  (func $_ZN95_$LT$std..panicking..begin_panic_handler..FormatStringPayload$u20$as$u20$core..fmt..Display$GT$3fmt17hd74758c7e313799bE (type 4) (param i32 i32) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.load
        i32.const -2147483648
        i32.eq
        br_if 0 (;@2;)
        local.get 1
        local.get 0
        i32.load offset=4
        local.get 0
        i32.load offset=8
        call $_ZN4core3fmt9Formatter9write_str17ha951e874492915b9E
        local.set 0
        br 1 (;@1;)
      end
      local.get 2
      i32.const 8
      i32.add
      i32.const 8
      i32.add
      local.get 0
      i32.load offset=12
      i32.load
      local.tee 0
      i32.const 8
      i32.add
      i64.load align=4
      i64.store
      local.get 2
      i32.const 8
      i32.add
      i32.const 16
      i32.add
      local.get 0
      i32.const 16
      i32.add
      i64.load align=4
      i64.store
      local.get 2
      local.get 0
      i64.load align=4
      i64.store offset=8
      local.get 1
      i32.load offset=20
      local.get 1
      i32.load offset=24
      local.get 2
      i32.const 8
      i32.add
      call $_ZN4core3fmt5write17hcf5d300c090957a7E
      local.set 0
    end
    local.get 2
    i32.const 32
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN99_$LT$std..panicking..begin_panic_handler..StaticStrPayload$u20$as$u20$core..panic..PanicPayload$GT$8take_box17h3a323dbea18dea96E (type 2) (param i32 i32)
    (local i32 i32)
    i32.const 0
    i32.load8_u offset=1058985
    drop
    local.get 1
    i32.load offset=4
    local.set 2
    local.get 1
    i32.load
    local.set 3
    block  ;; label = @1
      i32.const 8
      i32.const 4
      call $__rust_alloc
      local.tee 1
      br_if 0 (;@1;)
      i32.const 4
      i32.const 8
      call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
      unreachable
    end
    local.get 1
    local.get 2
    i32.store offset=4
    local.get 1
    local.get 3
    i32.store
    local.get 0
    i32.const 1054076
    i32.store offset=4
    local.get 0
    local.get 1
    i32.store)
  (func $_ZN99_$LT$std..panicking..begin_panic_handler..StaticStrPayload$u20$as$u20$core..panic..PanicPayload$GT$3get17h85e47ee294bc51aeE (type 2) (param i32 i32)
    local.get 0
    i32.const 1054076
    i32.store offset=4
    local.get 0
    local.get 1
    i32.store)
  (func $_ZN99_$LT$std..panicking..begin_panic_handler..StaticStrPayload$u20$as$u20$core..panic..PanicPayload$GT$6as_str17h12728f070b9d6551E (type 2) (param i32 i32)
    local.get 0
    local.get 1
    i64.load align=4
    i64.store)
  (func $_ZN92_$LT$std..panicking..begin_panic_handler..StaticStrPayload$u20$as$u20$core..fmt..Display$GT$3fmt17h7cb955cbcfed38e6E (type 4) (param i32 i32) (result i32)
    local.get 1
    local.get 0
    i32.load
    local.get 0
    i32.load offset=4
    call $_ZN4core3fmt9Formatter9write_str17ha951e874492915b9E)
  (func $_ZN3std9panicking20rust_panic_with_hook17hcc2503a68438883eE (type 7) (param i32 i32 i32 i32 i32)
    (local i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 5
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        i32.const 1
        call $_ZN3std9panicking11panic_count8increase17he00fd07a22feffa3E
        i32.const 255
        i32.and
        local.tee 6
        i32.const 2
        i32.eq
        br_if 0 (;@2;)
        local.get 6
        i32.const 1
        i32.and
        i32.eqz
        br_if 1 (;@1;)
        local.get 5
        i32.const 8
        i32.add
        local.get 0
        local.get 1
        i32.load offset=24
        call_indirect (type 2)
        unreachable
      end
      i32.const 0
      i32.load offset=1059092
      local.tee 6
      i32.const -1
      i32.le_s
      br_if 0 (;@1;)
      i32.const 0
      local.get 6
      i32.const 1
      i32.add
      i32.store offset=1059092
      block  ;; label = @2
        i32.const 0
        i32.load offset=1059096
        i32.eqz
        br_if 0 (;@2;)
        local.get 5
        local.get 0
        local.get 1
        i32.load offset=20
        call_indirect (type 2)
        local.get 5
        local.get 4
        i32.store8 offset=29
        local.get 5
        local.get 3
        i32.store8 offset=28
        local.get 5
        local.get 2
        i32.store offset=24
        local.get 5
        local.get 5
        i64.load
        i64.store offset=16 align=4
        i32.const 0
        i32.load offset=1059096
        local.get 5
        i32.const 16
        i32.add
        i32.const 0
        i32.load offset=1059100
        i32.load offset=20
        call_indirect (type 2)
        i32.const 0
        i32.load offset=1059092
        i32.const -1
        i32.add
        local.set 6
      end
      i32.const 0
      local.get 6
      i32.store offset=1059092
      i32.const 0
      i32.const 0
      i32.store8 offset=1059584
      local.get 3
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 1
      call $rust_panic
    end
    unreachable)
  (func $rust_panic (type 2) (param i32 i32)
    local.get 0
    local.get 1
    call $__rust_start_panic
    drop
    unreachable)
  (func $_ZN96_$LT$std..panicking..rust_panic_without_hook..RewrapBox$u20$as$u20$core..panic..PanicPayload$GT$8take_box17h659baeadd9cb1069E (type 2) (param i32 i32)
    (local i64)
    local.get 1
    i64.load align=4
    local.set 2
    local.get 1
    i32.const 1054188
    i32.store offset=4
    local.get 1
    i32.const 1
    i32.store
    local.get 0
    local.get 2
    i64.store)
  (func $_ZN96_$LT$std..panicking..rust_panic_without_hook..RewrapBox$u20$as$u20$core..panic..PanicPayload$GT$3get17hc81c0be5a0b540ffE (type 2) (param i32 i32)
    local.get 0
    local.get 1
    i64.load align=4
    i64.store)
  (func $_ZN89_$LT$std..panicking..rust_panic_without_hook..RewrapBox$u20$as$u20$core..fmt..Display$GT$3fmt17h8f45bfa2abb35ee5E (type 4) (param i32 i32) (result i32)
    local.get 1
    i32.const 1054148
    i32.const 12
    call $_ZN4core3fmt9Formatter9write_str17ha951e874492915b9E)
  (func $_ZN62_$LT$std..io..error..ErrorKind$u20$as$u20$core..fmt..Debug$GT$3fmt17hc9d9a0174449b0bfE (type 4) (param i32 i32) (result i32)
    local.get 1
    local.get 0
    i32.load8_u
    i32.const 2
    i32.shl
    local.tee 0
    i32.const 1055152
    i32.add
    i32.load
    local.get 0
    i32.const 1054984
    i32.add
    i32.load
    call $_ZN4core3fmt9Formatter9write_str17ha951e874492915b9E)
  (func $_ZN3std3sys3pal4wasm6common14abort_internal17h968250bab15ff6b6E (type 1)
    unreachable)
  (func $_ZN3std3sys3pal4wasm6thread6Thread3new17h7a2d93e59351b2efE (type 12) (param i32 i32 i32 i32)
    local.get 0
    i32.const 0
    i64.load offset=1053760
    i64.store align=4
    block  ;; label = @1
      local.get 3
      i32.load
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 0
      call_indirect (type 0)
    end
    block  ;; label = @1
      local.get 3
      i32.load offset=4
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 0
      local.get 3
      i32.load offset=8
      call $__rust_dealloc
    end)
  (func $_ZN3std3sys4sync7condvar10no_threads7Condvar4wait17hb91e5e8042bc81e5E (type 2) (param i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 0
    i32.store offset=24
    local.get 2
    i32.const 1
    i32.store offset=12
    local.get 2
    i32.const 1054804
    i32.store offset=8
    local.get 2
    i64.const 4
    i64.store offset=16 align=4
    local.get 2
    i32.const 8
    i32.add
    i32.const 1054852
    call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
    unreachable)
  (func $__rg_oom (type 2) (param i32 i32)
    (local i32)
    local.get 1
    local.get 0
    i32.const 0
    i32.load offset=1059088
    local.tee 2
    i32.const 23
    local.get 2
    select
    call_indirect (type 2)
    unreachable)
  (func $__rust_start_panic (type 4) (param i32 i32) (result i32)
    unreachable)
  (func $_ZN61_$LT$dlmalloc..sys..System$u20$as$u20$dlmalloc..Allocator$GT$5alloc17hd432c065eb8119e7E (type 3) (param i32 i32 i32)
    (local i32)
    local.get 2
    i32.const 16
    i32.shr_u
    memory.grow
    local.set 3
    local.get 0
    i32.const 0
    i32.store offset=8
    local.get 0
    i32.const 0
    local.get 2
    i32.const -65536
    i32.and
    local.get 3
    i32.const -1
    i32.eq
    local.tee 2
    select
    i32.store offset=4
    local.get 0
    i32.const 0
    local.get 3
    i32.const 16
    i32.shl
    local.get 2
    select
    i32.store)
  (func $_ZN69_$LT$core..alloc..layout..LayoutError$u20$as$u20$core..fmt..Debug$GT$3fmt17hdc389e2f810cfa03E (type 4) (param i32 i32) (result i32)
    local.get 1
    i32.const 1055320
    i32.const 11
    call $_ZN4core3fmt9Formatter9write_str17ha951e874492915b9E)
  (func $_ZN5alloc7raw_vec17capacity_overflow17h72b5bafc6c696719E (type 0) (param i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 1
    i32.const 0
    i32.store offset=24
    local.get 1
    i32.const 1
    i32.store offset=12
    local.get 1
    i32.const 1055348
    i32.store offset=8
    local.get 1
    i64.const 4
    i64.store offset=16 align=4
    local.get 1
    i32.const 8
    i32.add
    local.get 0
    call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
    unreachable)
  (func $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE (type 3) (param i32 i32 i32)
    block  ;; label = @1
      local.get 0
      br_if 0 (;@1;)
      local.get 2
      call $_ZN5alloc7raw_vec17capacity_overflow17h72b5bafc6c696719E
      unreachable
    end
    local.get 0
    local.get 1
    call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
    unreachable)
  (func $_ZN5alloc7raw_vec11finish_grow17h945d0bd8fd16d768E (type 12) (param i32 i32 i32 i32)
    (local i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        i32.const 0
        i32.lt_s
        br_if 0 (;@2;)
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 3
              i32.load offset=4
              i32.eqz
              br_if 0 (;@5;)
              block  ;; label = @6
                local.get 3
                i32.load offset=8
                local.tee 4
                br_if 0 (;@6;)
                block  ;; label = @7
                  local.get 2
                  br_if 0 (;@7;)
                  local.get 1
                  local.set 3
                  br 4 (;@3;)
                end
                i32.const 0
                i32.load8_u offset=1058985
                drop
                br 2 (;@4;)
              end
              local.get 3
              i32.load
              local.get 4
              local.get 1
              local.get 2
              call $__rust_realloc
              local.set 3
              br 2 (;@3;)
            end
            block  ;; label = @5
              local.get 2
              br_if 0 (;@5;)
              local.get 1
              local.set 3
              br 2 (;@3;)
            end
            i32.const 0
            i32.load8_u offset=1058985
            drop
          end
          local.get 2
          local.get 1
          call $__rust_alloc
          local.set 3
        end
        block  ;; label = @3
          local.get 3
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          local.get 2
          i32.store offset=8
          local.get 0
          local.get 3
          i32.store offset=4
          local.get 0
          i32.const 0
          i32.store
          return
        end
        local.get 0
        local.get 2
        i32.store offset=8
        local.get 0
        local.get 1
        i32.store offset=4
        br 1 (;@1;)
      end
      local.get 0
      i32.const 0
      i32.store offset=4
    end
    local.get 0
    i32.const 1
    i32.store)
  (func $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E (type 2) (param i32 i32)
    local.get 1
    local.get 0
    call $__rust_alloc_error_handler
    unreachable)
  (func $_ZN5alloc3ffi5c_str7CString19_from_vec_unchecked17hddc42bd4ebb2f1c3E (type 2) (param i32 i32)
    (local i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      local.get 1
      i32.load
      local.tee 3
      local.get 1
      i32.load offset=8
      local.tee 4
      i32.ne
      br_if 0 (;@1;)
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 4
            i32.const -1
            i32.ne
            br_if 0 (;@4;)
            i32.const 0
            local.set 5
            br 1 (;@3;)
          end
          i32.const 0
          local.set 5
          block  ;; label = @4
            local.get 4
            i32.const 1
            i32.add
            local.tee 3
            i32.const 0
            i32.ge_s
            br_if 0 (;@4;)
            br 1 (;@3;)
          end
          block  ;; label = @4
            block  ;; label = @5
              local.get 4
              br_if 0 (;@5;)
              i32.const 0
              local.set 5
              br 1 (;@4;)
            end
            local.get 2
            local.get 4
            i32.store offset=28
            local.get 2
            local.get 1
            i32.load offset=4
            i32.store offset=20
            i32.const 1
            local.set 5
          end
          local.get 2
          local.get 5
          i32.store offset=24
          local.get 2
          i32.const 8
          i32.add
          i32.const 1
          local.get 3
          local.get 2
          i32.const 20
          i32.add
          call $_ZN5alloc7raw_vec11finish_grow17h945d0bd8fd16d768E
          local.get 2
          i32.load offset=8
          i32.const 1
          i32.ne
          br_if 1 (;@2;)
          local.get 2
          i32.load offset=16
          local.set 1
          local.get 2
          i32.load offset=12
          local.set 5
        end
        local.get 5
        local.get 1
        i32.const 1055380
        call $_ZN5alloc7raw_vec12handle_error17h48c301ced15e16eeE
        unreachable
      end
      local.get 2
      i32.load offset=12
      local.set 5
      local.get 1
      local.get 3
      i32.store
      local.get 1
      local.get 5
      i32.store offset=4
    end
    local.get 1
    local.get 4
    i32.const 1
    i32.add
    local.tee 5
    i32.store offset=8
    local.get 1
    i32.load offset=4
    local.tee 1
    local.get 4
    i32.add
    i32.const 0
    i32.store8
    block  ;; label = @1
      block  ;; label = @2
        local.get 3
        local.get 5
        i32.gt_u
        br_if 0 (;@2;)
        local.get 1
        local.set 4
        br 1 (;@1;)
      end
      block  ;; label = @2
        local.get 5
        br_if 0 (;@2;)
        i32.const 1
        local.set 4
        local.get 1
        local.get 3
        i32.const 1
        call $__rust_dealloc
        br 1 (;@1;)
      end
      local.get 1
      local.get 3
      i32.const 1
      local.get 5
      call $__rust_realloc
      local.tee 4
      br_if 0 (;@1;)
      i32.const 1
      local.get 5
      call $_ZN5alloc5alloc18handle_alloc_error17hdf585cf4bb08d046E
      unreachable
    end
    local.get 0
    local.get 5
    i32.store offset=4
    local.get 0
    local.get 4
    i32.store
    local.get 2
    i32.const 32
    i32.add
    global.set $__stack_pointer)
  (func $_ZN5alloc4sync32arcinner_layout_for_value_layout17hea65de36431e6179E (type 3) (param i32 i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    block  ;; label = @1
      local.get 1
      i32.const 7
      i32.add
      i32.const 0
      local.get 1
      i32.sub
      i32.and
      local.get 2
      i32.add
      local.tee 2
      i32.const -2147483648
      local.get 1
      i32.const 4
      local.get 1
      i32.const 4
      i32.gt_u
      select
      local.tee 1
      i32.sub
      i32.gt_u
      br_if 0 (;@1;)
      local.get 0
      local.get 1
      i32.store
      local.get 0
      local.get 1
      local.get 2
      i32.add
      i32.const -1
      i32.add
      i32.const 0
      local.get 1
      i32.sub
      i32.and
      i32.store offset=4
      local.get 3
      i32.const 16
      i32.add
      global.set $__stack_pointer
      return
    end
    i32.const 1055412
    i32.const 43
    local.get 3
    i32.const 15
    i32.add
    i32.const 1055396
    i32.const 1055472
    call $_ZN4core6result13unwrap_failed17h89eac97f11bebdf4E
    unreachable)
  (func $_ZN4core9panicking18panic_bounds_check17h6c9fc24fb71a0cb6E (type 3) (param i32 i32 i32)
    (local i32 i64)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 3
    local.get 1
    i32.store offset=4
    local.get 3
    local.get 0
    i32.store
    local.get 3
    i32.const 2
    i32.store offset=12
    local.get 3
    i32.const 1055644
    i32.store offset=8
    local.get 3
    i64.const 2
    i64.store offset=20 align=4
    local.get 3
    i32.const 22
    i64.extend_i32_u
    i64.const 32
    i64.shl
    local.tee 4
    local.get 3
    i64.extend_i32_u
    i64.or
    i64.store offset=40
    local.get 3
    local.get 4
    local.get 3
    i32.const 4
    i32.add
    i64.extend_i32_u
    i64.or
    i64.store offset=32
    local.get 3
    local.get 3
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 3
    i32.const 8
    i32.add
    local.get 2
    call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
    unreachable)
  (func $_ZN4core5slice5index24slice_end_index_len_fail17h07937a589bfe269aE (type 3) (param i32 i32 i32)
    local.get 0
    local.get 1
    local.get 2
    call $_ZN4core5slice5index24slice_end_index_len_fail8do_panic7runtime17h40abf6316be1d38dE
    unreachable)
  (func $_ZN4core3fmt9Formatter3pad17h869c031e37b3eedeE (type 6) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32)
    local.get 0
    i32.load offset=8
    local.set 3
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.load
        local.tee 4
        br_if 0 (;@2;)
        local.get 3
        i32.const 1
        i32.and
        i32.eqz
        br_if 1 (;@1;)
      end
      block  ;; label = @2
        local.get 3
        i32.const 1
        i32.and
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        local.get 2
        i32.add
        local.set 5
        block  ;; label = @3
          block  ;; label = @4
            local.get 0
            i32.load offset=12
            local.tee 6
            br_if 0 (;@4;)
            i32.const 0
            local.set 7
            local.get 1
            local.set 8
            br 1 (;@3;)
          end
          i32.const 0
          local.set 7
          local.get 1
          local.set 8
          loop  ;; label = @4
            local.get 8
            local.tee 3
            local.get 5
            i32.eq
            br_if 2 (;@2;)
            block  ;; label = @5
              block  ;; label = @6
                local.get 3
                i32.load8_s
                local.tee 8
                i32.const -1
                i32.le_s
                br_if 0 (;@6;)
                local.get 3
                i32.const 1
                i32.add
                local.set 8
                br 1 (;@5;)
              end
              block  ;; label = @6
                local.get 8
                i32.const -32
                i32.ge_u
                br_if 0 (;@6;)
                local.get 3
                i32.const 2
                i32.add
                local.set 8
                br 1 (;@5;)
              end
              block  ;; label = @6
                local.get 8
                i32.const -16
                i32.ge_u
                br_if 0 (;@6;)
                local.get 3
                i32.const 3
                i32.add
                local.set 8
                br 1 (;@5;)
              end
              local.get 3
              i32.const 4
              i32.add
              local.set 8
            end
            local.get 8
            local.get 3
            i32.sub
            local.get 7
            i32.add
            local.set 7
            local.get 6
            i32.const -1
            i32.add
            local.tee 6
            br_if 0 (;@4;)
          end
        end
        local.get 8
        local.get 5
        i32.eq
        br_if 0 (;@2;)
        block  ;; label = @3
          local.get 8
          i32.load8_s
          local.tee 3
          i32.const -1
          i32.gt_s
          br_if 0 (;@3;)
          local.get 3
          i32.const -32
          i32.lt_u
          drop
        end
        block  ;; label = @3
          block  ;; label = @4
            local.get 7
            i32.eqz
            br_if 0 (;@4;)
            block  ;; label = @5
              local.get 7
              local.get 2
              i32.lt_u
              br_if 0 (;@5;)
              local.get 7
              local.get 2
              i32.eq
              br_if 1 (;@4;)
              i32.const 0
              local.set 3
              br 2 (;@3;)
            end
            local.get 1
            local.get 7
            i32.add
            i32.load8_s
            i32.const -64
            i32.ge_s
            br_if 0 (;@4;)
            i32.const 0
            local.set 3
            br 1 (;@3;)
          end
          local.get 1
          local.set 3
        end
        local.get 7
        local.get 2
        local.get 3
        select
        local.set 2
        local.get 3
        local.get 1
        local.get 3
        select
        local.set 1
      end
      block  ;; label = @2
        local.get 4
        br_if 0 (;@2;)
        local.get 0
        i32.load offset=20
        local.get 1
        local.get 2
        local.get 0
        i32.load offset=24
        i32.load offset=12
        call_indirect (type 6)
        return
      end
      local.get 0
      i32.load offset=4
      local.set 4
      block  ;; label = @2
        block  ;; label = @3
          local.get 2
          i32.const 16
          i32.lt_u
          br_if 0 (;@3;)
          local.get 1
          local.get 2
          call $_ZN4core3str5count14do_count_chars17hdc74440e30b08b44E
          local.set 3
          br 1 (;@2;)
        end
        block  ;; label = @3
          local.get 2
          br_if 0 (;@3;)
          i32.const 0
          local.set 3
          br 1 (;@2;)
        end
        local.get 2
        i32.const 3
        i32.and
        local.set 6
        block  ;; label = @3
          block  ;; label = @4
            local.get 2
            i32.const 4
            i32.ge_u
            br_if 0 (;@4;)
            i32.const 0
            local.set 3
            i32.const 0
            local.set 7
            br 1 (;@3;)
          end
          local.get 2
          i32.const 12
          i32.and
          local.set 5
          i32.const 0
          local.set 3
          i32.const 0
          local.set 7
          loop  ;; label = @4
            local.get 3
            local.get 1
            local.get 7
            i32.add
            local.tee 8
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 8
            i32.const 1
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 8
            i32.const 2
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 8
            i32.const 3
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.set 3
            local.get 5
            local.get 7
            i32.const 4
            i32.add
            local.tee 7
            i32.ne
            br_if 0 (;@4;)
          end
        end
        local.get 6
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        local.get 7
        i32.add
        local.set 8
        loop  ;; label = @3
          local.get 3
          local.get 8
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.set 3
          local.get 8
          i32.const 1
          i32.add
          local.set 8
          local.get 6
          i32.const -1
          i32.add
          local.tee 6
          br_if 0 (;@3;)
        end
      end
      block  ;; label = @2
        block  ;; label = @3
          local.get 4
          local.get 3
          i32.le_u
          br_if 0 (;@3;)
          local.get 4
          local.get 3
          i32.sub
          local.set 5
          i32.const 0
          local.set 3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                local.get 0
                i32.load8_u offset=32
                br_table 2 (;@4;) 0 (;@6;) 1 (;@5;) 2 (;@4;) 2 (;@4;)
              end
              local.get 5
              local.set 3
              i32.const 0
              local.set 5
              br 1 (;@4;)
            end
            local.get 5
            i32.const 1
            i32.shr_u
            local.set 3
            local.get 5
            i32.const 1
            i32.add
            i32.const 1
            i32.shr_u
            local.set 5
          end
          local.get 3
          i32.const 1
          i32.add
          local.set 3
          local.get 0
          i32.load offset=16
          local.set 6
          local.get 0
          i32.load offset=24
          local.set 8
          local.get 0
          i32.load offset=20
          local.set 7
          loop  ;; label = @4
            local.get 3
            i32.const -1
            i32.add
            local.tee 3
            i32.eqz
            br_if 2 (;@2;)
            local.get 7
            local.get 6
            local.get 8
            i32.load offset=16
            call_indirect (type 4)
            i32.eqz
            br_if 0 (;@4;)
          end
          i32.const 1
          return
        end
        local.get 0
        i32.load offset=20
        local.get 1
        local.get 2
        local.get 0
        i32.load offset=24
        i32.load offset=12
        call_indirect (type 6)
        return
      end
      block  ;; label = @2
        local.get 7
        local.get 1
        local.get 2
        local.get 8
        i32.load offset=12
        call_indirect (type 6)
        i32.eqz
        br_if 0 (;@2;)
        i32.const 1
        return
      end
      i32.const 0
      local.set 3
      loop  ;; label = @2
        block  ;; label = @3
          local.get 5
          local.get 3
          i32.ne
          br_if 0 (;@3;)
          local.get 5
          local.get 5
          i32.lt_u
          return
        end
        local.get 3
        i32.const 1
        i32.add
        local.set 3
        local.get 7
        local.get 6
        local.get 8
        i32.load offset=16
        call_indirect (type 4)
        i32.eqz
        br_if 0 (;@2;)
      end
      local.get 3
      i32.const -1
      i32.add
      local.get 5
      i32.lt_u
      return
    end
    local.get 0
    i32.load offset=20
    local.get 1
    local.get 2
    local.get 0
    i32.load offset=24
    i32.load offset=12
    call_indirect (type 6))
  (func $_ZN4core9panicking5panic17hb20c9056d85d5b5eE (type 3) (param i32 i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 3
    i32.const 0
    i32.store offset=16
    local.get 3
    i32.const 1
    i32.store offset=4
    local.get 3
    i64.const 4
    i64.store offset=8 align=4
    local.get 3
    local.get 1
    i32.store offset=28
    local.get 3
    local.get 0
    i32.store offset=24
    local.get 3
    local.get 3
    i32.const 24
    i32.add
    i32.store
    local.get 3
    local.get 2
    call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
    unreachable)
  (func $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E (type 2) (param i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 1
    i32.store16 offset=12
    local.get 2
    local.get 1
    i32.store offset=8
    local.get 2
    local.get 0
    i32.store offset=4
    local.get 2
    i32.const 4
    i32.add
    call $rust_begin_unwind
    unreachable)
  (func $_ZN4core3fmt5write17hcf5d300c090957a7E (type 6) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 3
    i32.const 3
    i32.store8 offset=44
    local.get 3
    i32.const 32
    i32.store offset=28
    i32.const 0
    local.set 4
    local.get 3
    i32.const 0
    i32.store offset=40
    local.get 3
    local.get 1
    i32.store offset=36
    local.get 3
    local.get 0
    i32.store offset=32
    local.get 3
    i32.const 0
    i32.store offset=20
    local.get 3
    i32.const 0
    i32.store offset=12
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 2
              i32.load offset=16
              local.tee 5
              br_if 0 (;@5;)
              local.get 2
              i32.load offset=12
              local.tee 0
              i32.eqz
              br_if 1 (;@4;)
              local.get 2
              i32.load offset=8
              local.tee 1
              local.get 0
              i32.const 3
              i32.shl
              i32.add
              local.set 6
              local.get 0
              i32.const -1
              i32.add
              i32.const 536870911
              i32.and
              i32.const 1
              i32.add
              local.set 4
              local.get 2
              i32.load
              local.set 0
              loop  ;; label = @6
                block  ;; label = @7
                  local.get 0
                  i32.const 4
                  i32.add
                  i32.load
                  local.tee 7
                  i32.eqz
                  br_if 0 (;@7;)
                  local.get 3
                  i32.load offset=32
                  local.get 0
                  i32.load
                  local.get 7
                  local.get 3
                  i32.load offset=36
                  i32.load offset=12
                  call_indirect (type 6)
                  br_if 4 (;@3;)
                end
                local.get 1
                i32.load
                local.get 3
                i32.const 12
                i32.add
                local.get 1
                i32.load offset=4
                call_indirect (type 4)
                br_if 3 (;@3;)
                local.get 0
                i32.const 8
                i32.add
                local.set 0
                local.get 1
                i32.const 8
                i32.add
                local.tee 1
                local.get 6
                i32.ne
                br_if 0 (;@6;)
                br 2 (;@4;)
              end
            end
            local.get 2
            i32.load offset=20
            local.tee 1
            i32.eqz
            br_if 0 (;@4;)
            local.get 1
            i32.const 5
            i32.shl
            local.set 8
            local.get 1
            i32.const -1
            i32.add
            i32.const 134217727
            i32.and
            i32.const 1
            i32.add
            local.set 4
            local.get 2
            i32.load offset=8
            local.set 9
            local.get 2
            i32.load
            local.set 0
            i32.const 0
            local.set 7
            loop  ;; label = @5
              block  ;; label = @6
                local.get 0
                i32.const 4
                i32.add
                i32.load
                local.tee 1
                i32.eqz
                br_if 0 (;@6;)
                local.get 3
                i32.load offset=32
                local.get 0
                i32.load
                local.get 1
                local.get 3
                i32.load offset=36
                i32.load offset=12
                call_indirect (type 6)
                br_if 3 (;@3;)
              end
              local.get 3
              local.get 5
              local.get 7
              i32.add
              local.tee 1
              i32.const 16
              i32.add
              i32.load
              i32.store offset=28
              local.get 3
              local.get 1
              i32.const 28
              i32.add
              i32.load8_u
              i32.store8 offset=44
              local.get 3
              local.get 1
              i32.const 24
              i32.add
              i32.load
              i32.store offset=40
              local.get 1
              i32.const 12
              i32.add
              i32.load
              local.set 6
              i32.const 0
              local.set 10
              i32.const 0
              local.set 11
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 1
                    i32.const 8
                    i32.add
                    i32.load
                    br_table 1 (;@7;) 0 (;@8;) 2 (;@6;) 1 (;@7;)
                  end
                  local.get 6
                  i32.const 3
                  i32.shl
                  local.set 12
                  i32.const 0
                  local.set 11
                  local.get 9
                  local.get 12
                  i32.add
                  local.tee 12
                  i32.load
                  br_if 1 (;@6;)
                  local.get 12
                  i32.load offset=4
                  local.set 6
                end
                i32.const 1
                local.set 11
              end
              local.get 3
              local.get 6
              i32.store offset=16
              local.get 3
              local.get 11
              i32.store offset=12
              local.get 1
              i32.const 4
              i32.add
              i32.load
              local.set 6
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 1
                    i32.load
                    br_table 1 (;@7;) 0 (;@8;) 2 (;@6;) 1 (;@7;)
                  end
                  local.get 6
                  i32.const 3
                  i32.shl
                  local.set 11
                  local.get 9
                  local.get 11
                  i32.add
                  local.tee 11
                  i32.load
                  br_if 1 (;@6;)
                  local.get 11
                  i32.load offset=4
                  local.set 6
                end
                i32.const 1
                local.set 10
              end
              local.get 3
              local.get 6
              i32.store offset=24
              local.get 3
              local.get 10
              i32.store offset=20
              local.get 9
              local.get 1
              i32.const 20
              i32.add
              i32.load
              i32.const 3
              i32.shl
              i32.add
              local.tee 1
              i32.load
              local.get 3
              i32.const 12
              i32.add
              local.get 1
              i32.load offset=4
              call_indirect (type 4)
              br_if 2 (;@3;)
              local.get 0
              i32.const 8
              i32.add
              local.set 0
              local.get 8
              local.get 7
              i32.const 32
              i32.add
              local.tee 7
              i32.ne
              br_if 0 (;@5;)
            end
          end
          local.get 4
          local.get 2
          i32.load offset=4
          i32.ge_u
          br_if 1 (;@2;)
          local.get 3
          i32.load offset=32
          local.get 2
          i32.load
          local.get 4
          i32.const 3
          i32.shl
          i32.add
          local.tee 1
          i32.load
          local.get 1
          i32.load offset=4
          local.get 3
          i32.load offset=36
          i32.load offset=12
          call_indirect (type 6)
          i32.eqz
          br_if 1 (;@2;)
        end
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      local.set 1
    end
    local.get 3
    i32.const 48
    i32.add
    global.set $__stack_pointer
    local.get 1)
  (func $_ZN71_$LT$core..ops..range..Range$LT$Idx$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h4e93df056c33b284E (type 4) (param i32 i32) (result i32)
    (local i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 128
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 1
            i32.load offset=28
            local.tee 3
            i32.const 16
            i32.and
            br_if 0 (;@4;)
            local.get 3
            i32.const 32
            i32.and
            br_if 1 (;@3;)
            i32.const 1
            local.set 3
            local.get 0
            i32.load
            i32.const 1
            local.get 1
            call $_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17h4f3209f6e643fb87E
            i32.eqz
            br_if 2 (;@2;)
            br 3 (;@1;)
          end
          local.get 0
          i32.load
          local.set 3
          i32.const 0
          local.set 4
          loop  ;; label = @4
            local.get 2
            local.get 4
            i32.add
            i32.const 127
            i32.add
            local.get 3
            i32.const 15
            i32.and
            local.tee 5
            i32.const 48
            i32.or
            local.get 5
            i32.const 87
            i32.add
            local.get 5
            i32.const 10
            i32.lt_u
            select
            i32.store8
            local.get 4
            i32.const -1
            i32.add
            local.set 4
            local.get 3
            i32.const 16
            i32.lt_u
            local.set 5
            local.get 3
            i32.const 4
            i32.shr_u
            local.set 3
            local.get 5
            i32.eqz
            br_if 0 (;@4;)
          end
          i32.const 1
          local.set 3
          local.get 1
          i32.const 1
          i32.const 1055871
          i32.const 2
          local.get 2
          local.get 4
          i32.add
          i32.const 128
          i32.add
          i32.const 0
          local.get 4
          i32.sub
          call $_ZN4core3fmt9Formatter12pad_integral17h121008db9b1c3bbbE
          i32.eqz
          br_if 1 (;@2;)
          br 2 (;@1;)
        end
        local.get 0
        i32.load
        local.set 3
        i32.const 0
        local.set 4
        loop  ;; label = @3
          local.get 2
          local.get 4
          i32.add
          i32.const 127
          i32.add
          local.get 3
          i32.const 15
          i32.and
          local.tee 5
          i32.const 48
          i32.or
          local.get 5
          i32.const 55
          i32.add
          local.get 5
          i32.const 10
          i32.lt_u
          select
          i32.store8
          local.get 4
          i32.const -1
          i32.add
          local.set 4
          local.get 3
          i32.const 15
          i32.gt_u
          local.set 5
          local.get 3
          i32.const 4
          i32.shr_u
          local.set 3
          local.get 5
          br_if 0 (;@3;)
        end
        i32.const 1
        local.set 3
        local.get 1
        i32.const 1
        i32.const 1055871
        i32.const 2
        local.get 2
        local.get 4
        i32.add
        i32.const 128
        i32.add
        i32.const 0
        local.get 4
        i32.sub
        call $_ZN4core3fmt9Formatter12pad_integral17h121008db9b1c3bbbE
        br_if 1 (;@1;)
      end
      i32.const 1
      local.set 3
      local.get 1
      i32.load offset=20
      i32.const 1055489
      i32.const 2
      local.get 1
      i32.load offset=24
      i32.load offset=12
      call_indirect (type 6)
      br_if 0 (;@1;)
      block  ;; label = @2
        block  ;; label = @3
          local.get 1
          i32.load offset=28
          local.tee 3
          i32.const 16
          i32.and
          br_if 0 (;@3;)
          local.get 3
          i32.const 32
          i32.and
          br_if 1 (;@2;)
          local.get 0
          i32.load offset=4
          i32.const 1
          local.get 1
          call $_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17h4f3209f6e643fb87E
          local.set 3
          br 2 (;@1;)
        end
        local.get 0
        i32.load offset=4
        local.set 3
        i32.const 0
        local.set 4
        loop  ;; label = @3
          local.get 2
          local.get 4
          i32.add
          i32.const 127
          i32.add
          local.get 3
          i32.const 15
          i32.and
          local.tee 5
          i32.const 48
          i32.or
          local.get 5
          i32.const 87
          i32.add
          local.get 5
          i32.const 10
          i32.lt_u
          select
          i32.store8
          local.get 4
          i32.const -1
          i32.add
          local.set 4
          local.get 3
          i32.const 15
          i32.gt_u
          local.set 5
          local.get 3
          i32.const 4
          i32.shr_u
          local.set 3
          local.get 5
          br_if 0 (;@3;)
        end
        local.get 1
        i32.const 1
        i32.const 1055871
        i32.const 2
        local.get 2
        local.get 4
        i32.add
        i32.const 128
        i32.add
        i32.const 0
        local.get 4
        i32.sub
        call $_ZN4core3fmt9Formatter12pad_integral17h121008db9b1c3bbbE
        local.set 3
        br 1 (;@1;)
      end
      local.get 0
      i32.load offset=4
      local.set 3
      i32.const 0
      local.set 4
      loop  ;; label = @2
        local.get 2
        local.get 4
        i32.add
        i32.const 127
        i32.add
        local.get 3
        i32.const 15
        i32.and
        local.tee 5
        i32.const 48
        i32.or
        local.get 5
        i32.const 55
        i32.add
        local.get 5
        i32.const 10
        i32.lt_u
        select
        i32.store8
        local.get 4
        i32.const -1
        i32.add
        local.set 4
        local.get 3
        i32.const 15
        i32.gt_u
        local.set 5
        local.get 3
        i32.const 4
        i32.shr_u
        local.set 3
        local.get 5
        br_if 0 (;@2;)
      end
      local.get 1
      i32.const 1
      i32.const 1055871
      i32.const 2
      local.get 2
      local.get 4
      i32.add
      i32.const 128
      i32.add
      i32.const 0
      local.get 4
      i32.sub
      call $_ZN4core3fmt9Formatter12pad_integral17h121008db9b1c3bbbE
      local.set 3
    end
    local.get 2
    i32.const 128
    i32.add
    global.set $__stack_pointer
    local.get 3)
  (func $_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17h4f3209f6e643fb87E (type 6) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    i32.const 10
    local.set 4
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.const 10000
        i32.ge_u
        br_if 0 (;@2;)
        local.get 0
        local.set 5
        br 1 (;@1;)
      end
      i32.const 10
      local.set 4
      loop  ;; label = @2
        local.get 3
        i32.const 6
        i32.add
        local.get 4
        i32.add
        local.tee 6
        i32.const -4
        i32.add
        local.get 0
        local.get 0
        i32.const 10000
        i32.div_u
        local.tee 5
        i32.const 10000
        i32.mul
        i32.sub
        local.tee 7
        i32.const 65535
        i32.and
        i32.const 100
        i32.div_u
        local.tee 8
        i32.const 1
        i32.shl
        i32.const 1055873
        i32.add
        i32.load16_u align=1
        i32.store16 align=1
        local.get 6
        i32.const -2
        i32.add
        local.get 7
        local.get 8
        i32.const 100
        i32.mul
        i32.sub
        i32.const 65535
        i32.and
        i32.const 1
        i32.shl
        i32.const 1055873
        i32.add
        i32.load16_u align=1
        i32.store16 align=1
        local.get 4
        i32.const -4
        i32.add
        local.set 4
        local.get 0
        i32.const 99999999
        i32.gt_u
        local.set 6
        local.get 5
        local.set 0
        local.get 6
        br_if 0 (;@2;)
      end
    end
    block  ;; label = @1
      block  ;; label = @2
        local.get 5
        i32.const 99
        i32.gt_u
        br_if 0 (;@2;)
        local.get 5
        local.set 0
        br 1 (;@1;)
      end
      local.get 3
      i32.const 6
      i32.add
      local.get 4
      i32.const -2
      i32.add
      local.tee 4
      i32.add
      local.get 5
      local.get 5
      i32.const 65535
      i32.and
      i32.const 100
      i32.div_u
      local.tee 0
      i32.const 100
      i32.mul
      i32.sub
      i32.const 65535
      i32.and
      i32.const 1
      i32.shl
      i32.const 1055873
      i32.add
      i32.load16_u align=1
      i32.store16 align=1
    end
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.const 10
        i32.lt_u
        br_if 0 (;@2;)
        local.get 3
        i32.const 6
        i32.add
        local.get 4
        i32.const -2
        i32.add
        local.tee 4
        i32.add
        local.get 0
        i32.const 1
        i32.shl
        i32.const 1055873
        i32.add
        i32.load16_u align=1
        i32.store16 align=1
        br 1 (;@1;)
      end
      local.get 3
      i32.const 6
      i32.add
      local.get 4
      i32.const -1
      i32.add
      local.tee 4
      i32.add
      local.get 0
      i32.const 48
      i32.or
      i32.store8
    end
    local.get 2
    local.get 1
    i32.const 1
    i32.const 0
    local.get 3
    i32.const 6
    i32.add
    local.get 4
    i32.add
    i32.const 10
    local.get 4
    i32.sub
    call $_ZN4core3fmt9Formatter12pad_integral17h121008db9b1c3bbbE
    local.set 0
    local.get 3
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN4core3fmt9Formatter12pad_integral17h121008db9b1c3bbbE (type 17) (param i32 i32 i32 i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        br_if 0 (;@2;)
        local.get 5
        i32.const 1
        i32.add
        local.set 6
        local.get 0
        i32.load offset=28
        local.set 7
        i32.const 45
        local.set 8
        br 1 (;@1;)
      end
      i32.const 43
      i32.const 1114112
      local.get 0
      i32.load offset=28
      local.tee 7
      i32.const 1
      i32.and
      local.tee 1
      select
      local.set 8
      local.get 1
      local.get 5
      i32.add
      local.set 6
    end
    block  ;; label = @1
      block  ;; label = @2
        local.get 7
        i32.const 4
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        local.set 2
        br 1 (;@1;)
      end
      block  ;; label = @2
        block  ;; label = @3
          local.get 3
          i32.const 16
          i32.lt_u
          br_if 0 (;@3;)
          local.get 2
          local.get 3
          call $_ZN4core3str5count14do_count_chars17hdc74440e30b08b44E
          local.set 1
          br 1 (;@2;)
        end
        block  ;; label = @3
          local.get 3
          br_if 0 (;@3;)
          i32.const 0
          local.set 1
          br 1 (;@2;)
        end
        local.get 3
        i32.const 3
        i32.and
        local.set 9
        block  ;; label = @3
          block  ;; label = @4
            local.get 3
            i32.const 4
            i32.ge_u
            br_if 0 (;@4;)
            i32.const 0
            local.set 1
            i32.const 0
            local.set 10
            br 1 (;@3;)
          end
          local.get 3
          i32.const 12
          i32.and
          local.set 11
          i32.const 0
          local.set 1
          i32.const 0
          local.set 10
          loop  ;; label = @4
            local.get 1
            local.get 2
            local.get 10
            i32.add
            local.tee 12
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 1
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 2
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 3
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.set 1
            local.get 11
            local.get 10
            i32.const 4
            i32.add
            local.tee 10
            i32.ne
            br_if 0 (;@4;)
          end
        end
        local.get 9
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 10
        i32.add
        local.set 12
        loop  ;; label = @3
          local.get 1
          local.get 12
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.set 1
          local.get 12
          i32.const 1
          i32.add
          local.set 12
          local.get 9
          i32.const -1
          i32.add
          local.tee 9
          br_if 0 (;@3;)
        end
      end
      local.get 1
      local.get 6
      i32.add
      local.set 6
    end
    block  ;; label = @1
      local.get 0
      i32.load
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 0
        i32.load offset=20
        local.tee 1
        local.get 0
        i32.load offset=24
        local.tee 12
        local.get 8
        local.get 2
        local.get 3
        call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17h04f3d1814dfa2468E
        i32.eqz
        br_if 0 (;@2;)
        i32.const 1
        return
      end
      local.get 1
      local.get 4
      local.get 5
      local.get 12
      i32.load offset=12
      call_indirect (type 6)
      return
    end
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 0
            i32.load offset=4
            local.tee 1
            local.get 6
            i32.gt_u
            br_if 0 (;@4;)
            local.get 0
            i32.load offset=20
            local.tee 1
            local.get 0
            i32.load offset=24
            local.tee 12
            local.get 8
            local.get 2
            local.get 3
            call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17h04f3d1814dfa2468E
            i32.eqz
            br_if 1 (;@3;)
            i32.const 1
            return
          end
          local.get 7
          i32.const 8
          i32.and
          i32.eqz
          br_if 1 (;@2;)
          local.get 0
          i32.load offset=16
          local.set 9
          local.get 0
          i32.const 48
          i32.store offset=16
          local.get 0
          i32.load8_u offset=32
          local.set 7
          i32.const 1
          local.set 11
          local.get 0
          i32.const 1
          i32.store8 offset=32
          local.get 0
          i32.load offset=20
          local.tee 12
          local.get 0
          i32.load offset=24
          local.tee 10
          local.get 8
          local.get 2
          local.get 3
          call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17h04f3d1814dfa2468E
          br_if 2 (;@1;)
          local.get 1
          local.get 6
          i32.sub
          i32.const 1
          i32.add
          local.set 1
          block  ;; label = @4
            loop  ;; label = @5
              local.get 1
              i32.const -1
              i32.add
              local.tee 1
              i32.eqz
              br_if 1 (;@4;)
              local.get 12
              i32.const 48
              local.get 10
              i32.load offset=16
              call_indirect (type 4)
              i32.eqz
              br_if 0 (;@5;)
            end
            i32.const 1
            return
          end
          block  ;; label = @4
            local.get 12
            local.get 4
            local.get 5
            local.get 10
            i32.load offset=12
            call_indirect (type 6)
            i32.eqz
            br_if 0 (;@4;)
            i32.const 1
            return
          end
          local.get 0
          local.get 7
          i32.store8 offset=32
          local.get 0
          local.get 9
          i32.store offset=16
          i32.const 0
          return
        end
        local.get 1
        local.get 4
        local.get 5
        local.get 12
        i32.load offset=12
        call_indirect (type 6)
        local.set 11
        br 1 (;@1;)
      end
      local.get 1
      local.get 6
      i32.sub
      local.set 6
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 0
            i32.load8_u offset=32
            local.tee 1
            br_table 2 (;@2;) 0 (;@4;) 1 (;@3;) 0 (;@4;) 2 (;@2;)
          end
          local.get 6
          local.set 1
          i32.const 0
          local.set 6
          br 1 (;@2;)
        end
        local.get 6
        i32.const 1
        i32.shr_u
        local.set 1
        local.get 6
        i32.const 1
        i32.add
        i32.const 1
        i32.shr_u
        local.set 6
      end
      local.get 1
      i32.const 1
      i32.add
      local.set 1
      local.get 0
      i32.load offset=16
      local.set 9
      local.get 0
      i32.load offset=24
      local.set 12
      local.get 0
      i32.load offset=20
      local.set 10
      block  ;; label = @2
        loop  ;; label = @3
          local.get 1
          i32.const -1
          i32.add
          local.tee 1
          i32.eqz
          br_if 1 (;@2;)
          local.get 10
          local.get 9
          local.get 12
          i32.load offset=16
          call_indirect (type 4)
          i32.eqz
          br_if 0 (;@3;)
        end
        i32.const 1
        return
      end
      i32.const 1
      local.set 11
      local.get 10
      local.get 12
      local.get 8
      local.get 2
      local.get 3
      call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17h04f3d1814dfa2468E
      br_if 0 (;@1;)
      local.get 10
      local.get 4
      local.get 5
      local.get 12
      i32.load offset=12
      call_indirect (type 6)
      br_if 0 (;@1;)
      i32.const 0
      local.set 1
      loop  ;; label = @2
        block  ;; label = @3
          local.get 6
          local.get 1
          i32.ne
          br_if 0 (;@3;)
          local.get 6
          local.get 6
          i32.lt_u
          return
        end
        local.get 1
        i32.const 1
        i32.add
        local.set 1
        local.get 10
        local.get 9
        local.get 12
        i32.load offset=16
        call_indirect (type 4)
        i32.eqz
        br_if 0 (;@2;)
      end
      local.get 1
      i32.const -1
      i32.add
      local.get 6
      i32.lt_u
      return
    end
    local.get 11)
  (func $_ZN63_$LT$core..cell..BorrowMutError$u20$as$u20$core..fmt..Debug$GT$3fmt17h85594013aacd41e5E (type 4) (param i32 i32) (result i32)
    local.get 1
    i32.load offset=20
    i32.const 1055507
    i32.const 14
    local.get 1
    i32.load offset=24
    i32.load offset=12
    call_indirect (type 6))
  (func $_ZN4core4cell22panic_already_borrowed17h0fba8746ae6f569aE (type 0) (param i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 1
    i32.const 1
    i32.store offset=12
    local.get 1
    i32.const 1055540
    i32.store offset=8
    local.get 1
    i64.const 1
    i64.store offset=20 align=4
    local.get 1
    i32.const 55
    i64.extend_i32_u
    i64.const 32
    i64.shl
    local.get 1
    i32.const 47
    i32.add
    i64.extend_i32_u
    i64.or
    i64.store offset=32
    local.get 1
    local.get 1
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 1
    i32.const 8
    i32.add
    local.get 0
    call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
    unreachable)
  (func $_ZN4core4char7methods22_$LT$impl$u20$char$GT$16escape_debug_ext17h5376b048d63917f7E (type 3) (param i32 i32 i32)
    (local i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      block  ;; label = @10
                        block  ;; label = @11
                          block  ;; label = @12
                            local.get 1
                            br_table 6 (;@6;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 2 (;@10;) 4 (;@8;) 1 (;@11;) 1 (;@11;) 3 (;@9;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 8 (;@4;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 1 (;@11;) 7 (;@5;) 0 (;@12;)
                          end
                          local.get 1
                          i32.const 92
                          i32.eq
                          br_if 4 (;@7;)
                        end
                        local.get 1
                        i32.const 768
                        i32.lt_u
                        br_if 7 (;@3;)
                        local.get 2
                        i32.const 1
                        i32.and
                        i32.eqz
                        br_if 7 (;@3;)
                        local.get 1
                        call $_ZN4core7unicode12unicode_data15grapheme_extend11lookup_slow17ha2724bdda0376e6fE
                        i32.eqz
                        br_if 7 (;@3;)
                        local.get 3
                        i32.const 0
                        i32.store8 offset=10
                        local.get 3
                        i32.const 0
                        i32.store16 offset=8
                        local.get 3
                        local.get 1
                        i32.const 20
                        i32.shr_u
                        i32.const 1055491
                        i32.add
                        i32.load8_u
                        i32.store8 offset=11
                        local.get 3
                        local.get 1
                        i32.const 4
                        i32.shr_u
                        i32.const 15
                        i32.and
                        i32.const 1055491
                        i32.add
                        i32.load8_u
                        i32.store8 offset=15
                        local.get 3
                        local.get 1
                        i32.const 8
                        i32.shr_u
                        i32.const 15
                        i32.and
                        i32.const 1055491
                        i32.add
                        i32.load8_u
                        i32.store8 offset=14
                        local.get 3
                        local.get 1
                        i32.const 12
                        i32.shr_u
                        i32.const 15
                        i32.and
                        i32.const 1055491
                        i32.add
                        i32.load8_u
                        i32.store8 offset=13
                        local.get 3
                        local.get 1
                        i32.const 16
                        i32.shr_u
                        i32.const 15
                        i32.and
                        i32.const 1055491
                        i32.add
                        i32.load8_u
                        i32.store8 offset=12
                        local.get 3
                        i32.const 8
                        i32.add
                        local.get 1
                        i32.const 1
                        i32.or
                        i32.clz
                        i32.const 2
                        i32.shr_u
                        local.tee 2
                        i32.add
                        local.tee 4
                        i32.const 123
                        i32.store8
                        local.get 4
                        i32.const -1
                        i32.add
                        i32.const 117
                        i32.store8
                        local.get 3
                        i32.const 8
                        i32.add
                        local.get 2
                        i32.const -2
                        i32.add
                        local.tee 2
                        i32.add
                        i32.const 92
                        i32.store8
                        local.get 3
                        i32.const 8
                        i32.add
                        i32.const 8
                        i32.add
                        local.tee 4
                        local.get 1
                        i32.const 15
                        i32.and
                        i32.const 1055491
                        i32.add
                        i32.load8_u
                        i32.store8
                        local.get 0
                        i32.const 10
                        i32.store8 offset=11
                        local.get 0
                        local.get 2
                        i32.store8 offset=10
                        local.get 0
                        local.get 3
                        i64.load offset=8 align=4
                        i64.store align=4
                        local.get 3
                        i32.const 125
                        i32.store8 offset=17
                        local.get 0
                        i32.const 8
                        i32.add
                        local.get 4
                        i32.load16_u
                        i32.store16
                        br 9 (;@1;)
                      end
                      local.get 0
                      i32.const 512
                      i32.store16 offset=10
                      local.get 0
                      i64.const 0
                      i64.store offset=2 align=2
                      local.get 0
                      i32.const 29788
                      i32.store16
                      br 8 (;@1;)
                    end
                    local.get 0
                    i32.const 512
                    i32.store16 offset=10
                    local.get 0
                    i64.const 0
                    i64.store offset=2 align=2
                    local.get 0
                    i32.const 29276
                    i32.store16
                    br 7 (;@1;)
                  end
                  local.get 0
                  i32.const 512
                  i32.store16 offset=10
                  local.get 0
                  i64.const 0
                  i64.store offset=2 align=2
                  local.get 0
                  i32.const 28252
                  i32.store16
                  br 6 (;@1;)
                end
                local.get 0
                i32.const 512
                i32.store16 offset=10
                local.get 0
                i64.const 0
                i64.store offset=2 align=2
                local.get 0
                i32.const 23644
                i32.store16
                br 5 (;@1;)
              end
              local.get 0
              i32.const 512
              i32.store16 offset=10
              local.get 0
              i64.const 0
              i64.store offset=2 align=2
              local.get 0
              i32.const 12380
              i32.store16
              br 4 (;@1;)
            end
            local.get 2
            i32.const 256
            i32.and
            i32.eqz
            br_if 1 (;@3;)
            local.get 0
            i32.const 512
            i32.store16 offset=10
            local.get 0
            i64.const 0
            i64.store offset=2 align=2
            local.get 0
            i32.const 10076
            i32.store16
            br 3 (;@1;)
          end
          local.get 2
          i32.const 65536
          i32.and
          br_if 1 (;@2;)
        end
        block  ;; label = @3
          local.get 1
          call $_ZN4core7unicode9printable12is_printable17h602f5441a1dc806fE
          br_if 0 (;@3;)
          local.get 3
          i32.const 0
          i32.store8 offset=22
          local.get 3
          i32.const 0
          i32.store16 offset=20
          local.get 3
          local.get 1
          i32.const 20
          i32.shr_u
          i32.const 1055491
          i32.add
          i32.load8_u
          i32.store8 offset=23
          local.get 3
          local.get 1
          i32.const 4
          i32.shr_u
          i32.const 15
          i32.and
          i32.const 1055491
          i32.add
          i32.load8_u
          i32.store8 offset=27
          local.get 3
          local.get 1
          i32.const 8
          i32.shr_u
          i32.const 15
          i32.and
          i32.const 1055491
          i32.add
          i32.load8_u
          i32.store8 offset=26
          local.get 3
          local.get 1
          i32.const 12
          i32.shr_u
          i32.const 15
          i32.and
          i32.const 1055491
          i32.add
          i32.load8_u
          i32.store8 offset=25
          local.get 3
          local.get 1
          i32.const 16
          i32.shr_u
          i32.const 15
          i32.and
          i32.const 1055491
          i32.add
          i32.load8_u
          i32.store8 offset=24
          local.get 3
          i32.const 20
          i32.add
          local.get 1
          i32.const 1
          i32.or
          i32.clz
          i32.const 2
          i32.shr_u
          local.tee 2
          i32.add
          local.tee 4
          i32.const 123
          i32.store8
          local.get 4
          i32.const -1
          i32.add
          i32.const 117
          i32.store8
          local.get 3
          i32.const 20
          i32.add
          local.get 2
          i32.const -2
          i32.add
          local.tee 2
          i32.add
          i32.const 92
          i32.store8
          local.get 3
          i32.const 20
          i32.add
          i32.const 8
          i32.add
          local.tee 4
          local.get 1
          i32.const 15
          i32.and
          i32.const 1055491
          i32.add
          i32.load8_u
          i32.store8
          local.get 0
          i32.const 10
          i32.store8 offset=11
          local.get 0
          local.get 2
          i32.store8 offset=10
          local.get 0
          local.get 3
          i64.load offset=20 align=4
          i64.store align=4
          local.get 3
          i32.const 125
          i32.store8 offset=29
          local.get 0
          i32.const 8
          i32.add
          local.get 4
          i32.load16_u
          i32.store16
          br 2 (;@1;)
        end
        local.get 0
        local.get 1
        i32.store offset=4
        local.get 0
        i32.const 128
        i32.store8
        br 1 (;@1;)
      end
      local.get 0
      i32.const 512
      i32.store16 offset=10
      local.get 0
      i64.const 0
      i64.store offset=2 align=2
      local.get 0
      i32.const 8796
      i32.store16
    end
    local.get 3
    i32.const 32
    i32.add
    global.set $__stack_pointer)
  (func $_ZN4core7unicode12unicode_data15grapheme_extend11lookup_slow17ha2724bdda0376e6fE (type 5) (param i32) (result i32)
    (local i32 i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        i32.const 0
        i32.const 17
        local.get 0
        i32.const 71727
        i32.lt_u
        select
        local.tee 1
        local.get 1
        i32.const 8
        i32.or
        local.tee 1
        local.get 1
        i32.const 2
        i32.shl
        i32.const 1058096
        i32.add
        i32.load
        i32.const 11
        i32.shl
        local.get 0
        i32.const 11
        i32.shl
        local.tee 1
        i32.gt_u
        select
        local.tee 2
        local.get 2
        i32.const 4
        i32.or
        local.tee 2
        local.get 2
        i32.const 2
        i32.shl
        i32.const 1058096
        i32.add
        i32.load
        i32.const 11
        i32.shl
        local.get 1
        i32.gt_u
        select
        local.tee 2
        local.get 2
        i32.const 2
        i32.or
        local.tee 2
        local.get 2
        i32.const 2
        i32.shl
        i32.const 1058096
        i32.add
        i32.load
        i32.const 11
        i32.shl
        local.get 1
        i32.gt_u
        select
        local.tee 2
        local.get 2
        i32.const 1
        i32.add
        local.tee 2
        local.get 2
        i32.const 2
        i32.shl
        i32.const 1058096
        i32.add
        i32.load
        i32.const 11
        i32.shl
        local.get 1
        i32.gt_u
        select
        local.tee 2
        local.get 2
        i32.const 1
        i32.add
        local.tee 2
        local.get 2
        i32.const 2
        i32.shl
        i32.const 1058096
        i32.add
        i32.load
        i32.const 11
        i32.shl
        local.get 1
        i32.gt_u
        select
        local.tee 2
        i32.const 2
        i32.shl
        i32.const 1058096
        i32.add
        i32.load
        i32.const 11
        i32.shl
        local.tee 3
        local.get 1
        i32.eq
        local.get 3
        local.get 1
        i32.lt_u
        i32.add
        local.get 2
        i32.add
        local.tee 2
        i32.const 33
        i32.gt_u
        br_if 0 (;@2;)
        local.get 2
        i32.const 2
        i32.shl
        i32.const 1058096
        i32.add
        local.tee 3
        i32.load
        i32.const 21
        i32.shr_u
        local.set 1
        i32.const 751
        local.set 4
        block  ;; label = @3
          block  ;; label = @4
            local.get 2
            i32.const 33
            i32.eq
            br_if 0 (;@4;)
            local.get 3
            i32.load offset=4
            i32.const 21
            i32.shr_u
            local.set 4
            local.get 2
            br_if 0 (;@4;)
            i32.const 0
            local.set 2
            br 1 (;@3;)
          end
          local.get 2
          i32.const 2
          i32.shl
          i32.const 1058092
          i32.add
          i32.load
          i32.const 2097151
          i32.and
          local.set 2
        end
        block  ;; label = @3
          local.get 4
          local.get 1
          i32.const -1
          i32.xor
          i32.add
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          local.get 2
          i32.sub
          local.set 3
          local.get 1
          i32.const 751
          local.get 1
          i32.const 751
          i32.gt_u
          select
          local.set 0
          local.get 4
          i32.const -1
          i32.add
          local.set 4
          i32.const 0
          local.set 2
          loop  ;; label = @4
            local.get 0
            local.get 1
            i32.eq
            br_if 3 (;@1;)
            local.get 2
            local.get 1
            i32.const 1058232
            i32.add
            i32.load8_u
            i32.add
            local.tee 2
            local.get 3
            i32.gt_u
            br_if 1 (;@3;)
            local.get 4
            local.get 1
            i32.const 1
            i32.add
            local.tee 1
            i32.ne
            br_if 0 (;@4;)
          end
          local.get 4
          local.set 1
        end
        local.get 1
        i32.const 1
        i32.and
        return
      end
      local.get 2
      i32.const 34
      i32.const 1057944
      call $_ZN4core9panicking18panic_bounds_check17h6c9fc24fb71a0cb6E
      unreachable
    end
    local.get 0
    i32.const 751
    i32.const 1057960
    call $_ZN4core9panicking18panic_bounds_check17h6c9fc24fb71a0cb6E
    unreachable)
  (func $_ZN4core7unicode9printable12is_printable17h602f5441a1dc806fE (type 5) (param i32) (result i32)
    block  ;; label = @1
      local.get 0
      i32.const 32
      i32.ge_u
      br_if 0 (;@1;)
      i32.const 0
      return
    end
    block  ;; label = @1
      local.get 0
      i32.const 127
      i32.ge_u
      br_if 0 (;@1;)
      i32.const 1
      return
    end
    block  ;; label = @1
      local.get 0
      i32.const 65536
      i32.lt_u
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 0
        i32.const 131072
        i32.lt_u
        br_if 0 (;@2;)
        local.get 0
        i32.const 2097120
        i32.and
        i32.const 173792
        i32.ne
        local.get 0
        i32.const 2097150
        i32.and
        i32.const 178206
        i32.ne
        i32.and
        local.get 0
        i32.const -177984
        i32.add
        i32.const -6
        i32.lt_u
        i32.and
        local.get 0
        i32.const -183984
        i32.add
        i32.const -14
        i32.lt_u
        i32.and
        local.get 0
        i32.const -191472
        i32.add
        i32.const -15
        i32.lt_u
        i32.and
        local.get 0
        i32.const -194560
        i32.add
        i32.const -2466
        i32.lt_u
        i32.and
        local.get 0
        i32.const -196608
        i32.add
        i32.const -1506
        i32.lt_u
        i32.and
        local.get 0
        i32.const -201552
        i32.add
        i32.const -5
        i32.lt_u
        i32.and
        local.get 0
        i32.const -917760
        i32.add
        i32.const -712016
        i32.lt_u
        i32.and
        local.get 0
        i32.const 918000
        i32.lt_u
        i32.and
        return
      end
      local.get 0
      i32.const 1056460
      i32.const 44
      i32.const 1056548
      i32.const 208
      i32.const 1056756
      i32.const 486
      call $_ZN4core7unicode9printable5check17h54417f65c5968d49E
      return
    end
    local.get 0
    i32.const 1057242
    i32.const 40
    i32.const 1057322
    i32.const 290
    i32.const 1057612
    i32.const 297
    call $_ZN4core7unicode9printable5check17h54417f65c5968d49E)
  (func $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17he1d3bba66865ae66E (type 4) (param i32 i32) (result i32)
    local.get 0
    i32.load
    i32.const 1
    local.get 1
    call $_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17h4f3209f6e643fb87E)
  (func $_ZN4core3fmt8builders11DebugStruct5field17h30cdc1b516ecb392E (type 13) (param i32 i32 i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i64)
    global.get $__stack_pointer
    i32.const 64
    i32.sub
    local.tee 5
    global.set $__stack_pointer
    i32.const 1
    local.set 6
    block  ;; label = @1
      local.get 0
      i32.load8_u offset=4
      br_if 0 (;@1;)
      local.get 0
      i32.load8_u offset=5
      local.set 7
      block  ;; label = @2
        local.get 0
        i32.load
        local.tee 8
        i32.load offset=28
        local.tee 9
        i32.const 4
        i32.and
        br_if 0 (;@2;)
        i32.const 1
        local.set 6
        local.get 8
        i32.load offset=20
        i32.const 1055855
        i32.const 1055852
        local.get 7
        i32.const 1
        i32.and
        local.tee 7
        select
        i32.const 2
        i32.const 3
        local.get 7
        select
        local.get 8
        i32.load offset=24
        i32.load offset=12
        call_indirect (type 6)
        br_if 1 (;@1;)
        local.get 8
        i32.load offset=20
        local.get 1
        local.get 2
        local.get 8
        i32.load offset=24
        i32.load offset=12
        call_indirect (type 6)
        br_if 1 (;@1;)
        local.get 8
        i32.load offset=20
        i32.const 1055804
        i32.const 2
        local.get 8
        i32.load offset=24
        i32.load offset=12
        call_indirect (type 6)
        br_if 1 (;@1;)
        local.get 3
        local.get 8
        local.get 4
        i32.load offset=12
        call_indirect (type 4)
        local.set 6
        br 1 (;@1;)
      end
      i32.const 1
      local.set 6
      block  ;; label = @2
        local.get 7
        i32.const 1
        i32.and
        br_if 0 (;@2;)
        local.get 8
        i32.load offset=20
        i32.const 1055857
        i32.const 3
        local.get 8
        i32.load offset=24
        i32.load offset=12
        call_indirect (type 6)
        br_if 1 (;@1;)
        local.get 8
        i32.load offset=28
        local.set 9
      end
      i32.const 1
      local.set 6
      local.get 5
      i32.const 1
      i32.store8 offset=27
      local.get 5
      local.get 8
      i64.load offset=20 align=4
      i64.store offset=12 align=4
      local.get 5
      i32.const 1055824
      i32.store offset=52
      local.get 5
      local.get 5
      i32.const 27
      i32.add
      i32.store offset=20
      local.get 5
      local.get 8
      i64.load offset=8 align=4
      i64.store offset=36 align=4
      local.get 8
      i64.load align=4
      local.set 10
      local.get 5
      local.get 9
      i32.store offset=56
      local.get 5
      local.get 8
      i32.load offset=16
      i32.store offset=44
      local.get 5
      local.get 8
      i32.load8_u offset=32
      i32.store8 offset=60
      local.get 5
      local.get 10
      i64.store offset=28 align=4
      local.get 5
      local.get 5
      i32.const 12
      i32.add
      i32.store offset=48
      local.get 5
      i32.const 12
      i32.add
      local.get 1
      local.get 2
      call $_ZN68_$LT$core..fmt..builders..PadAdapter$u20$as$u20$core..fmt..Write$GT$9write_str17h50c1b995f7ff56e5E
      br_if 0 (;@1;)
      local.get 5
      i32.const 12
      i32.add
      i32.const 1055804
      i32.const 2
      call $_ZN68_$LT$core..fmt..builders..PadAdapter$u20$as$u20$core..fmt..Write$GT$9write_str17h50c1b995f7ff56e5E
      br_if 0 (;@1;)
      local.get 3
      local.get 5
      i32.const 28
      i32.add
      local.get 4
      i32.load offset=12
      call_indirect (type 4)
      br_if 0 (;@1;)
      local.get 5
      i32.load offset=48
      i32.const 1055860
      i32.const 2
      local.get 5
      i32.load offset=52
      i32.load offset=12
      call_indirect (type 6)
      local.set 6
    end
    local.get 0
    i32.const 1
    i32.store8 offset=5
    local.get 0
    local.get 6
    i32.store8 offset=4
    local.get 5
    i32.const 64
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN4core3fmt3num3imp51_$LT$impl$u20$core..fmt..Display$u20$for$u20$u8$GT$3fmt17he72bf047e6e0be63E (type 4) (param i32 i32) (result i32)
    (local i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 0
            i32.load8_u
            local.tee 3
            i32.const 100
            i32.lt_u
            br_if 0 (;@4;)
            local.get 2
            local.get 3
            local.get 3
            i32.const 100
            i32.div_u
            local.tee 4
            i32.const 100
            i32.mul
            i32.sub
            i32.const 255
            i32.and
            i32.const 1
            i32.shl
            i32.const 1055873
            i32.add
            i32.load16_u align=1
            i32.store16 offset=14 align=1
            i32.const 0
            local.set 0
            br 1 (;@3;)
          end
          i32.const 2
          local.set 0
          local.get 3
          i32.const 10
          i32.ge_u
          br_if 1 (;@2;)
          local.get 3
          local.set 4
        end
        local.get 2
        i32.const 13
        i32.add
        local.get 0
        i32.add
        local.get 4
        i32.const 48
        i32.or
        i32.store8
        br 1 (;@1;)
      end
      i32.const 1
      local.set 0
      local.get 2
      local.get 3
      i32.const 1
      i32.shl
      i32.const 1055873
      i32.add
      i32.load16_u align=1
      i32.store16 offset=14 align=1
    end
    local.get 1
    i32.const 1
    i32.const 1
    i32.const 0
    local.get 2
    i32.const 13
    i32.add
    local.get 0
    i32.add
    local.get 0
    i32.const 3
    i32.xor
    call $_ZN4core3fmt9Formatter12pad_integral17h121008db9b1c3bbbE
    local.set 0
    local.get 2
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN4core6result13unwrap_failed17h89eac97f11bebdf4E (type 7) (param i32 i32 i32 i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 64
    i32.sub
    local.tee 5
    global.set $__stack_pointer
    local.get 5
    local.get 1
    i32.store offset=12
    local.get 5
    local.get 0
    i32.store offset=8
    local.get 5
    local.get 3
    i32.store offset=20
    local.get 5
    local.get 2
    i32.store offset=16
    local.get 5
    i32.const 2
    i32.store offset=28
    local.get 5
    i32.const 1055808
    i32.store offset=24
    local.get 5
    i64.const 2
    i64.store offset=36 align=4
    local.get 5
    i32.const 56
    i64.extend_i32_u
    i64.const 32
    i64.shl
    local.get 5
    i32.const 16
    i32.add
    i64.extend_i32_u
    i64.or
    i64.store offset=56
    local.get 5
    i32.const 57
    i64.extend_i32_u
    i64.const 32
    i64.shl
    local.get 5
    i32.const 8
    i32.add
    i64.extend_i32_u
    i64.or
    i64.store offset=48
    local.get 5
    local.get 5
    i32.const 48
    i32.add
    i32.store offset=32
    local.get 5
    i32.const 24
    i32.add
    local.get 4
    call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
    unreachable)
  (func $_ZN4core5slice5index22slice_index_order_fail17hce8d8012711d5a00E (type 3) (param i32 i32 i32)
    local.get 0
    local.get 1
    local.get 2
    call $_ZN4core5slice5index22slice_index_order_fail8do_panic7runtime17h3eb440503b756a77E
    unreachable)
  (func $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE (type 0) (param i32)
    i32.const 1055549
    i32.const 43
    local.get 0
    call $_ZN4core9panicking5panic17hb20c9056d85d5b5eE
    unreachable)
  (func $_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h8ef67506d1a750e3E (type 4) (param i32 i32) (result i32)
    local.get 1
    local.get 0
    i32.load
    local.get 0
    i32.load offset=4
    call $_ZN4core3fmt9Formatter3pad17h869c031e37b3eedeE)
  (func $_ZN4core3fmt3num53_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$i32$GT$3fmt17hee4425a51b6b9c20E (type 4) (param i32 i32) (result i32)
    (local i32 i32 i32)
    global.get $__stack_pointer
    i32.const 128
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 0
    i32.load
    local.set 0
    i32.const 0
    local.set 3
    loop  ;; label = @1
      local.get 2
      local.get 3
      i32.add
      i32.const 127
      i32.add
      local.get 0
      i32.const 15
      i32.and
      local.tee 4
      i32.const 48
      i32.or
      local.get 4
      i32.const 87
      i32.add
      local.get 4
      i32.const 10
      i32.lt_u
      select
      i32.store8
      local.get 3
      i32.const -1
      i32.add
      local.set 3
      local.get 0
      i32.const 15
      i32.gt_u
      local.set 4
      local.get 0
      i32.const 4
      i32.shr_u
      local.set 0
      local.get 4
      br_if 0 (;@1;)
    end
    local.get 1
    i32.const 1
    i32.const 1055871
    i32.const 2
    local.get 2
    local.get 3
    i32.add
    i32.const 128
    i32.add
    i32.const 0
    local.get 3
    i32.sub
    call $_ZN4core3fmt9Formatter12pad_integral17h121008db9b1c3bbbE
    local.set 0
    local.get 2
    i32.const 128
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN4core9panicking19assert_failed_inner17he4920e028524a869E (type 18) (param i32 i32 i32 i32 i32 i32 i32)
    (local i32 i64)
    global.get $__stack_pointer
    i32.const 112
    i32.sub
    local.tee 7
    global.set $__stack_pointer
    local.get 7
    local.get 2
    i32.store offset=12
    local.get 7
    local.get 1
    i32.store offset=8
    local.get 7
    local.get 4
    i32.store offset=20
    local.get 7
    local.get 3
    i32.store offset=16
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 0
            i32.const 255
            i32.and
            br_table 0 (;@4;) 1 (;@3;) 2 (;@2;) 0 (;@4;)
          end
          local.get 7
          i32.const 1055660
          i32.store offset=24
          i32.const 2
          local.set 2
          br 2 (;@1;)
        end
        local.get 7
        i32.const 1055662
        i32.store offset=24
        i32.const 2
        local.set 2
        br 1 (;@1;)
      end
      local.get 7
      i32.const 1055664
      i32.store offset=24
      i32.const 7
      local.set 2
    end
    local.get 7
    local.get 2
    i32.store offset=28
    block  ;; label = @1
      local.get 5
      i32.load
      br_if 0 (;@1;)
      local.get 7
      i32.const 3
      i32.store offset=92
      local.get 7
      i32.const 1055720
      i32.store offset=88
      local.get 7
      i64.const 3
      i64.store offset=100 align=4
      local.get 7
      i32.const 56
      i64.extend_i32_u
      i64.const 32
      i64.shl
      local.tee 8
      local.get 7
      i32.const 16
      i32.add
      i64.extend_i32_u
      i64.or
      i64.store offset=72
      local.get 7
      local.get 8
      local.get 7
      i32.const 8
      i32.add
      i64.extend_i32_u
      i64.or
      i64.store offset=64
      local.get 7
      i32.const 57
      i64.extend_i32_u
      i64.const 32
      i64.shl
      local.get 7
      i32.const 24
      i32.add
      i64.extend_i32_u
      i64.or
      i64.store offset=56
      local.get 7
      local.get 7
      i32.const 56
      i32.add
      i32.store offset=96
      local.get 7
      i32.const 88
      i32.add
      local.get 6
      call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
      unreachable
    end
    local.get 7
    i32.const 32
    i32.add
    i32.const 16
    i32.add
    local.get 5
    i32.const 16
    i32.add
    i64.load align=4
    i64.store
    local.get 7
    i32.const 32
    i32.add
    i32.const 8
    i32.add
    local.get 5
    i32.const 8
    i32.add
    i64.load align=4
    i64.store
    local.get 7
    local.get 5
    i64.load align=4
    i64.store offset=32
    local.get 7
    i32.const 4
    i32.store offset=92
    local.get 7
    i32.const 1055772
    i32.store offset=88
    local.get 7
    i64.const 4
    i64.store offset=100 align=4
    local.get 7
    i32.const 56
    i64.extend_i32_u
    i64.const 32
    i64.shl
    local.tee 8
    local.get 7
    i32.const 16
    i32.add
    i64.extend_i32_u
    i64.or
    i64.store offset=80
    local.get 7
    local.get 8
    local.get 7
    i32.const 8
    i32.add
    i64.extend_i32_u
    i64.or
    i64.store offset=72
    local.get 7
    i32.const 58
    i64.extend_i32_u
    i64.const 32
    i64.shl
    local.get 7
    i32.const 32
    i32.add
    i64.extend_i32_u
    i64.or
    i64.store offset=64
    local.get 7
    i32.const 57
    i64.extend_i32_u
    i64.const 32
    i64.shl
    local.get 7
    i32.const 24
    i32.add
    i64.extend_i32_u
    i64.or
    i64.store offset=56
    local.get 7
    local.get 7
    i32.const 56
    i32.add
    i32.store offset=96
    local.get 7
    i32.const 88
    i32.add
    local.get 6
    call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
    unreachable)
  (func $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hb26ba63d6930e1beE (type 4) (param i32 i32) (result i32)
    local.get 0
    i32.load
    local.get 1
    local.get 0
    i32.load offset=4
    i32.load offset=12
    call_indirect (type 4))
  (func $_ZN59_$LT$core..fmt..Arguments$u20$as$u20$core..fmt..Display$GT$3fmt17h710b3907e5e9fa00E (type 4) (param i32 i32) (result i32)
    local.get 1
    i32.load offset=20
    local.get 1
    i32.load offset=24
    local.get 0
    call $_ZN4core3fmt5write17hcf5d300c090957a7E)
  (func $_ZN68_$LT$core..fmt..builders..PadAdapter$u20$as$u20$core..fmt..Write$GT$9write_str17h50c1b995f7ff56e5E (type 6) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    local.get 1
    i32.const -1
    i32.add
    local.set 3
    local.get 0
    i32.load offset=4
    local.set 4
    local.get 0
    i32.load
    local.set 5
    local.get 0
    i32.load offset=8
    local.set 6
    i32.const 0
    local.set 7
    i32.const 0
    local.set 8
    i32.const 0
    local.set 9
    i32.const 0
    local.set 10
    block  ;; label = @1
      loop  ;; label = @2
        local.get 10
        i32.const 1
        i32.and
        br_if 1 (;@1;)
        block  ;; label = @3
          block  ;; label = @4
            local.get 9
            local.get 2
            i32.gt_u
            br_if 0 (;@4;)
            loop  ;; label = @5
              local.get 1
              local.get 9
              i32.add
              local.set 11
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      local.get 2
                      local.get 9
                      i32.sub
                      local.tee 12
                      i32.const 7
                      i32.gt_u
                      br_if 0 (;@9;)
                      local.get 2
                      local.get 9
                      i32.ne
                      br_if 1 (;@8;)
                      local.get 2
                      local.set 9
                      br 5 (;@4;)
                    end
                    block  ;; label = @9
                      block  ;; label = @10
                        local.get 11
                        i32.const 3
                        i32.add
                        i32.const -4
                        i32.and
                        local.tee 13
                        local.get 11
                        i32.sub
                        local.tee 14
                        i32.eqz
                        br_if 0 (;@10;)
                        i32.const 0
                        local.set 0
                        loop  ;; label = @11
                          local.get 11
                          local.get 0
                          i32.add
                          i32.load8_u
                          i32.const 10
                          i32.eq
                          br_if 5 (;@6;)
                          local.get 14
                          local.get 0
                          i32.const 1
                          i32.add
                          local.tee 0
                          i32.ne
                          br_if 0 (;@11;)
                        end
                        local.get 14
                        local.get 12
                        i32.const -8
                        i32.add
                        local.tee 15
                        i32.le_u
                        br_if 1 (;@9;)
                        br 3 (;@7;)
                      end
                      local.get 12
                      i32.const -8
                      i32.add
                      local.set 15
                    end
                    loop  ;; label = @9
                      i32.const 16843008
                      local.get 13
                      i32.load
                      local.tee 0
                      i32.const 168430090
                      i32.xor
                      i32.sub
                      local.get 0
                      i32.or
                      i32.const 16843008
                      local.get 13
                      i32.const 4
                      i32.add
                      i32.load
                      local.tee 0
                      i32.const 168430090
                      i32.xor
                      i32.sub
                      local.get 0
                      i32.or
                      i32.and
                      i32.const -2139062144
                      i32.and
                      i32.const -2139062144
                      i32.ne
                      br_if 2 (;@7;)
                      local.get 13
                      i32.const 8
                      i32.add
                      local.set 13
                      local.get 14
                      i32.const 8
                      i32.add
                      local.tee 14
                      local.get 15
                      i32.le_u
                      br_if 0 (;@9;)
                      br 2 (;@7;)
                    end
                  end
                  i32.const 0
                  local.set 0
                  loop  ;; label = @8
                    local.get 11
                    local.get 0
                    i32.add
                    i32.load8_u
                    i32.const 10
                    i32.eq
                    br_if 2 (;@6;)
                    local.get 12
                    local.get 0
                    i32.const 1
                    i32.add
                    local.tee 0
                    i32.ne
                    br_if 0 (;@8;)
                  end
                  local.get 2
                  local.set 9
                  br 3 (;@4;)
                end
                block  ;; label = @7
                  local.get 14
                  local.get 12
                  i32.ne
                  br_if 0 (;@7;)
                  local.get 2
                  local.set 9
                  br 3 (;@4;)
                end
                loop  ;; label = @7
                  block  ;; label = @8
                    local.get 11
                    local.get 14
                    i32.add
                    i32.load8_u
                    i32.const 10
                    i32.ne
                    br_if 0 (;@8;)
                    local.get 14
                    local.set 0
                    br 2 (;@6;)
                  end
                  local.get 12
                  local.get 14
                  i32.const 1
                  i32.add
                  local.tee 14
                  i32.ne
                  br_if 0 (;@7;)
                end
                local.get 2
                local.set 9
                br 2 (;@4;)
              end
              local.get 0
              local.get 9
              i32.add
              local.tee 14
              i32.const 1
              i32.add
              local.set 9
              block  ;; label = @6
                local.get 14
                local.get 2
                i32.ge_u
                br_if 0 (;@6;)
                local.get 11
                local.get 0
                i32.add
                i32.load8_u
                i32.const 10
                i32.ne
                br_if 0 (;@6;)
                local.get 9
                local.set 11
                local.get 9
                local.set 0
                br 3 (;@3;)
              end
              local.get 9
              local.get 2
              i32.le_u
              br_if 0 (;@5;)
            end
          end
          i32.const 1
          local.set 10
          local.get 8
          local.set 11
          local.get 2
          local.set 0
          local.get 8
          local.get 2
          i32.eq
          br_if 2 (;@1;)
        end
        block  ;; label = @3
          block  ;; label = @4
            local.get 6
            i32.load8_u
            i32.eqz
            br_if 0 (;@4;)
            local.get 5
            i32.const 1055848
            i32.const 4
            local.get 4
            i32.load offset=12
            call_indirect (type 6)
            br_if 1 (;@3;)
          end
          local.get 0
          local.get 8
          i32.sub
          local.set 13
          i32.const 0
          local.set 14
          block  ;; label = @4
            local.get 0
            local.get 8
            i32.eq
            br_if 0 (;@4;)
            local.get 3
            local.get 0
            i32.add
            i32.load8_u
            i32.const 10
            i32.eq
            local.set 14
          end
          local.get 1
          local.get 8
          i32.add
          local.set 0
          local.get 6
          local.get 14
          i32.store8
          local.get 11
          local.set 8
          local.get 5
          local.get 0
          local.get 13
          local.get 4
          i32.load offset=12
          call_indirect (type 6)
          i32.eqz
          br_if 1 (;@2;)
        end
      end
      i32.const 1
      local.set 7
    end
    local.get 7)
  (func $_ZN68_$LT$core..fmt..builders..PadAdapter$u20$as$u20$core..fmt..Write$GT$10write_char17habd2bb5515050163E (type 4) (param i32 i32) (result i32)
    (local i32 i32)
    local.get 0
    i32.load offset=4
    local.set 2
    local.get 0
    i32.load
    local.set 3
    block  ;; label = @1
      local.get 0
      i32.load offset=8
      local.tee 0
      i32.load8_u
      i32.eqz
      br_if 0 (;@1;)
      local.get 3
      i32.const 1055848
      i32.const 4
      local.get 2
      i32.load offset=12
      call_indirect (type 6)
      i32.eqz
      br_if 0 (;@1;)
      i32.const 1
      return
    end
    local.get 0
    local.get 1
    i32.const 10
    i32.eq
    i32.store8
    local.get 3
    local.get 1
    local.get 2
    i32.load offset=16
    call_indirect (type 4))
  (func $_ZN4core3fmt8builders11DebugStruct6finish17h02d6fd6087b36dfdE (type 5) (param i32) (result i32)
    (local i32 i32)
    local.get 0
    i32.load8_u offset=4
    local.tee 1
    local.set 2
    block  ;; label = @1
      local.get 0
      i32.load8_u offset=5
      i32.eqz
      br_if 0 (;@1;)
      i32.const 1
      local.set 2
      block  ;; label = @2
        local.get 1
        i32.const 1
        i32.and
        br_if 0 (;@2;)
        block  ;; label = @3
          local.get 0
          i32.load
          local.tee 2
          i32.load8_u offset=28
          i32.const 4
          i32.and
          br_if 0 (;@3;)
          local.get 2
          i32.load offset=20
          i32.const 1055863
          i32.const 2
          local.get 2
          i32.load offset=24
          i32.load offset=12
          call_indirect (type 6)
          local.set 2
          br 1 (;@2;)
        end
        local.get 2
        i32.load offset=20
        i32.const 1055862
        i32.const 1
        local.get 2
        i32.load offset=24
        i32.load offset=12
        call_indirect (type 6)
        local.set 2
      end
      local.get 0
      local.get 2
      i32.store8 offset=4
    end
    local.get 2
    i32.const 1
    i32.and)
  (func $_ZN4core3fmt8builders10DebugTuple5field17h16ef0971d382d024E (type 6) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i64)
    global.get $__stack_pointer
    i32.const 64
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 0
    i32.load
    local.set 4
    i32.const 1
    local.set 5
    block  ;; label = @1
      local.get 0
      i32.load8_u offset=8
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 0
        i32.load offset=4
        local.tee 6
        i32.load offset=28
        local.tee 7
        i32.const 4
        i32.and
        br_if 0 (;@2;)
        i32.const 1
        local.set 5
        local.get 6
        i32.load offset=20
        i32.const 1055855
        i32.const 1055865
        local.get 4
        select
        i32.const 2
        i32.const 1
        local.get 4
        select
        local.get 6
        i32.load offset=24
        i32.load offset=12
        call_indirect (type 6)
        br_if 1 (;@1;)
        local.get 1
        local.get 6
        local.get 2
        i32.load offset=12
        call_indirect (type 4)
        local.set 5
        br 1 (;@1;)
      end
      block  ;; label = @2
        local.get 4
        br_if 0 (;@2;)
        i32.const 1
        local.set 5
        local.get 6
        i32.load offset=20
        i32.const 1055866
        i32.const 2
        local.get 6
        i32.load offset=24
        i32.load offset=12
        call_indirect (type 6)
        br_if 1 (;@1;)
        local.get 6
        i32.load offset=28
        local.set 7
      end
      i32.const 1
      local.set 5
      local.get 3
      i32.const 1
      i32.store8 offset=27
      local.get 3
      local.get 6
      i64.load offset=20 align=4
      i64.store offset=12 align=4
      local.get 3
      i32.const 1055824
      i32.store offset=52
      local.get 3
      local.get 3
      i32.const 27
      i32.add
      i32.store offset=20
      local.get 3
      local.get 6
      i64.load offset=8 align=4
      i64.store offset=36 align=4
      local.get 6
      i64.load align=4
      local.set 8
      local.get 3
      local.get 7
      i32.store offset=56
      local.get 3
      local.get 6
      i32.load offset=16
      i32.store offset=44
      local.get 3
      local.get 6
      i32.load8_u offset=32
      i32.store8 offset=60
      local.get 3
      local.get 8
      i64.store offset=28 align=4
      local.get 3
      local.get 3
      i32.const 12
      i32.add
      i32.store offset=48
      local.get 1
      local.get 3
      i32.const 28
      i32.add
      local.get 2
      i32.load offset=12
      call_indirect (type 4)
      br_if 0 (;@1;)
      local.get 3
      i32.load offset=48
      i32.const 1055860
      i32.const 2
      local.get 3
      i32.load offset=52
      i32.load offset=12
      call_indirect (type 6)
      local.set 5
    end
    local.get 0
    local.get 5
    i32.store8 offset=8
    local.get 0
    local.get 4
    i32.const 1
    i32.add
    i32.store
    local.get 3
    i32.const 64
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN4core3fmt8builders10DebugTuple6finish17he75364125644b4c4E (type 5) (param i32) (result i32)
    (local i32 i32 i32)
    local.get 0
    i32.load8_u offset=8
    local.set 1
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.load
        local.tee 2
        br_if 0 (;@2;)
        local.get 1
        local.set 3
        br 1 (;@1;)
      end
      i32.const 1
      local.set 3
      block  ;; label = @2
        block  ;; label = @3
          local.get 1
          i32.const 1
          i32.and
          br_if 0 (;@3;)
          local.get 2
          i32.const 1
          i32.ne
          br_if 1 (;@2;)
          local.get 0
          i32.load8_u offset=9
          i32.eqz
          br_if 1 (;@2;)
          local.get 0
          i32.load offset=4
          local.tee 1
          i32.load8_u offset=28
          i32.const 4
          i32.and
          br_if 1 (;@2;)
          i32.const 1
          local.set 3
          local.get 1
          i32.load offset=20
          i32.const 1055868
          i32.const 1
          local.get 1
          i32.load offset=24
          i32.load offset=12
          call_indirect (type 6)
          i32.eqz
          br_if 1 (;@2;)
        end
        local.get 0
        local.get 3
        i32.store8 offset=8
        br 1 (;@1;)
      end
      local.get 0
      local.get 0
      i32.load offset=4
      local.tee 3
      i32.load offset=20
      i32.const 1055488
      i32.const 1
      local.get 3
      i32.load offset=24
      i32.load offset=12
      call_indirect (type 6)
      local.tee 3
      i32.store8 offset=8
    end
    local.get 3
    i32.const 1
    i32.and)
  (func $_ZN4core3fmt8builders8DebugSet5entry17h4dbe0c39ee61b1a5E (type 6) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i64)
    global.get $__stack_pointer
    i32.const 64
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    i32.const 1
    local.set 4
    block  ;; label = @1
      local.get 0
      i32.load8_u offset=4
      br_if 0 (;@1;)
      local.get 0
      i32.load8_u offset=5
      local.set 5
      block  ;; label = @2
        block  ;; label = @3
          local.get 0
          i32.load
          local.tee 6
          i32.load offset=28
          local.tee 7
          i32.const 4
          i32.and
          br_if 0 (;@3;)
          i32.const 1
          local.set 4
          local.get 5
          i32.const 1
          i32.and
          i32.eqz
          br_if 1 (;@2;)
          local.get 6
          i32.load offset=20
          i32.const 1055855
          i32.const 2
          local.get 6
          i32.load offset=24
          i32.load offset=12
          call_indirect (type 6)
          i32.eqz
          br_if 1 (;@2;)
          br 2 (;@1;)
        end
        i32.const 1
        local.set 4
        block  ;; label = @3
          local.get 5
          i32.const 1
          i32.and
          br_if 0 (;@3;)
          local.get 6
          i32.load offset=20
          i32.const 1055869
          i32.const 1
          local.get 6
          i32.load offset=24
          i32.load offset=12
          call_indirect (type 6)
          br_if 2 (;@1;)
          local.get 6
          i32.load offset=28
          local.set 7
        end
        i32.const 1
        local.set 4
        local.get 3
        i32.const 1
        i32.store8 offset=27
        local.get 3
        local.get 6
        i64.load offset=20 align=4
        i64.store offset=12 align=4
        local.get 3
        i32.const 1055824
        i32.store offset=52
        local.get 3
        local.get 3
        i32.const 27
        i32.add
        i32.store offset=20
        local.get 3
        local.get 6
        i64.load offset=8 align=4
        i64.store offset=36 align=4
        local.get 6
        i64.load align=4
        local.set 8
        local.get 3
        local.get 7
        i32.store offset=56
        local.get 3
        local.get 6
        i32.load offset=16
        i32.store offset=44
        local.get 3
        local.get 6
        i32.load8_u offset=32
        i32.store8 offset=60
        local.get 3
        local.get 8
        i64.store offset=28 align=4
        local.get 3
        local.get 3
        i32.const 12
        i32.add
        i32.store offset=48
        local.get 1
        local.get 3
        i32.const 28
        i32.add
        local.get 2
        i32.load offset=12
        call_indirect (type 4)
        br_if 1 (;@1;)
        local.get 3
        i32.load offset=48
        i32.const 1055860
        i32.const 2
        local.get 3
        i32.load offset=52
        i32.load offset=12
        call_indirect (type 6)
        local.set 4
        br 1 (;@1;)
      end
      local.get 1
      local.get 6
      local.get 2
      i32.load offset=12
      call_indirect (type 4)
      local.set 4
    end
    local.get 0
    i32.const 1
    i32.store8 offset=5
    local.get 0
    local.get 4
    i32.store8 offset=4
    local.get 3
    i32.const 64
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN4core3fmt8builders9DebugList6finish17h522bde96a5fb0888E (type 5) (param i32) (result i32)
    (local i32)
    i32.const 1
    local.set 1
    block  ;; label = @1
      local.get 0
      i32.load8_u offset=4
      br_if 0 (;@1;)
      local.get 0
      i32.load
      local.tee 1
      i32.load offset=20
      i32.const 1055870
      i32.const 1
      local.get 1
      i32.load offset=24
      i32.load offset=12
      call_indirect (type 6)
      local.set 1
    end
    local.get 0
    local.get 1
    i32.store8 offset=4
    local.get 1)
  (func $_ZN4core3fmt5Write9write_fmt17hca9ef98f6e4b87ecE (type 4) (param i32 i32) (result i32)
    local.get 0
    i32.const 1055824
    local.get 1
    call $_ZN4core3fmt5write17hcf5d300c090957a7E)
  (func $_ZN4core3str5count14do_count_chars17hdc74440e30b08b44E (type 4) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        local.get 0
        i32.const 3
        i32.add
        i32.const -4
        i32.and
        local.tee 2
        local.get 0
        i32.sub
        local.tee 3
        i32.lt_u
        br_if 0 (;@2;)
        local.get 1
        local.get 3
        i32.sub
        local.tee 4
        i32.const 4
        i32.lt_u
        br_if 0 (;@2;)
        local.get 4
        i32.const 3
        i32.and
        local.set 5
        i32.const 0
        local.set 6
        i32.const 0
        local.set 1
        block  ;; label = @3
          local.get 2
          local.get 0
          i32.eq
          local.tee 7
          br_if 0 (;@3;)
          i32.const 0
          local.set 1
          block  ;; label = @4
            block  ;; label = @5
              local.get 0
              local.get 2
              i32.sub
              local.tee 8
              i32.const -4
              i32.le_u
              br_if 0 (;@5;)
              i32.const 0
              local.set 9
              br 1 (;@4;)
            end
            i32.const 0
            local.set 9
            loop  ;; label = @5
              local.get 1
              local.get 0
              local.get 9
              i32.add
              local.tee 2
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 1
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 2
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 3
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.set 1
              local.get 9
              i32.const 4
              i32.add
              local.tee 9
              br_if 0 (;@5;)
            end
          end
          local.get 7
          br_if 0 (;@3;)
          local.get 0
          local.get 9
          i32.add
          local.set 2
          loop  ;; label = @4
            local.get 1
            local.get 2
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.set 1
            local.get 2
            i32.const 1
            i32.add
            local.set 2
            local.get 8
            i32.const 1
            i32.add
            local.tee 8
            br_if 0 (;@4;)
          end
        end
        local.get 0
        local.get 3
        i32.add
        local.set 0
        block  ;; label = @3
          local.get 5
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          local.get 4
          i32.const -4
          i32.and
          i32.add
          local.tee 2
          i32.load8_s
          i32.const -65
          i32.gt_s
          local.set 6
          local.get 5
          i32.const 1
          i32.eq
          br_if 0 (;@3;)
          local.get 6
          local.get 2
          i32.load8_s offset=1
          i32.const -65
          i32.gt_s
          i32.add
          local.set 6
          local.get 5
          i32.const 2
          i32.eq
          br_if 0 (;@3;)
          local.get 6
          local.get 2
          i32.load8_s offset=2
          i32.const -65
          i32.gt_s
          i32.add
          local.set 6
        end
        local.get 4
        i32.const 2
        i32.shr_u
        local.set 8
        local.get 6
        local.get 1
        i32.add
        local.set 3
        loop  ;; label = @3
          local.get 0
          local.set 4
          local.get 8
          i32.eqz
          br_if 2 (;@1;)
          local.get 8
          i32.const 192
          local.get 8
          i32.const 192
          i32.lt_u
          select
          local.tee 6
          i32.const 3
          i32.and
          local.set 7
          local.get 6
          i32.const 2
          i32.shl
          local.set 5
          i32.const 0
          local.set 2
          block  ;; label = @4
            local.get 8
            i32.const 4
            i32.lt_u
            br_if 0 (;@4;)
            local.get 4
            local.get 5
            i32.const 1008
            i32.and
            i32.add
            local.set 9
            i32.const 0
            local.set 2
            local.get 4
            local.set 1
            loop  ;; label = @5
              local.get 1
              i32.load offset=12
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.load offset=8
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.load offset=4
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 2
              i32.add
              i32.add
              i32.add
              i32.add
              local.set 2
              local.get 1
              i32.const 16
              i32.add
              local.tee 1
              local.get 9
              i32.ne
              br_if 0 (;@5;)
            end
          end
          local.get 8
          local.get 6
          i32.sub
          local.set 8
          local.get 4
          local.get 5
          i32.add
          local.set 0
          local.get 2
          i32.const 8
          i32.shr_u
          i32.const 16711935
          i32.and
          local.get 2
          i32.const 16711935
          i32.and
          i32.add
          i32.const 65537
          i32.mul
          i32.const 16
          i32.shr_u
          local.get 3
          i32.add
          local.set 3
          local.get 7
          i32.eqz
          br_if 0 (;@3;)
        end
        local.get 4
        local.get 6
        i32.const 252
        i32.and
        i32.const 2
        i32.shl
        i32.add
        local.tee 2
        i32.load
        local.tee 1
        i32.const -1
        i32.xor
        i32.const 7
        i32.shr_u
        local.get 1
        i32.const 6
        i32.shr_u
        i32.or
        i32.const 16843009
        i32.and
        local.set 1
        block  ;; label = @3
          local.get 7
          i32.const 1
          i32.eq
          br_if 0 (;@3;)
          local.get 2
          i32.load offset=4
          local.tee 0
          i32.const -1
          i32.xor
          i32.const 7
          i32.shr_u
          local.get 0
          i32.const 6
          i32.shr_u
          i32.or
          i32.const 16843009
          i32.and
          local.get 1
          i32.add
          local.set 1
          local.get 7
          i32.const 2
          i32.eq
          br_if 0 (;@3;)
          local.get 2
          i32.load offset=8
          local.tee 2
          i32.const -1
          i32.xor
          i32.const 7
          i32.shr_u
          local.get 2
          i32.const 6
          i32.shr_u
          i32.or
          i32.const 16843009
          i32.and
          local.get 1
          i32.add
          local.set 1
        end
        local.get 1
        i32.const 8
        i32.shr_u
        i32.const 459007
        i32.and
        local.get 1
        i32.const 16711935
        i32.and
        i32.add
        i32.const 65537
        i32.mul
        i32.const 16
        i32.shr_u
        local.get 3
        i32.add
        return
      end
      block  ;; label = @2
        local.get 1
        br_if 0 (;@2;)
        i32.const 0
        return
      end
      local.get 1
      i32.const 3
      i32.and
      local.set 9
      block  ;; label = @2
        block  ;; label = @3
          local.get 1
          i32.const 4
          i32.ge_u
          br_if 0 (;@3;)
          i32.const 0
          local.set 3
          i32.const 0
          local.set 2
          br 1 (;@2;)
        end
        local.get 1
        i32.const -4
        i32.and
        local.set 8
        i32.const 0
        local.set 3
        i32.const 0
        local.set 2
        loop  ;; label = @3
          local.get 3
          local.get 0
          local.get 2
          i32.add
          local.tee 1
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 1
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 2
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 3
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.set 3
          local.get 8
          local.get 2
          i32.const 4
          i32.add
          local.tee 2
          i32.ne
          br_if 0 (;@3;)
        end
      end
      local.get 9
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 2
      i32.add
      local.set 1
      loop  ;; label = @2
        local.get 3
        local.get 1
        i32.load8_s
        i32.const -65
        i32.gt_s
        i32.add
        local.set 3
        local.get 1
        i32.const 1
        i32.add
        local.set 1
        local.get 9
        i32.const -1
        i32.add
        local.tee 9
        br_if 0 (;@2;)
      end
    end
    local.get 3)
  (func $_ZN4core3fmt9Formatter12pad_integral12write_prefix17h04f3d1814dfa2468E (type 13) (param i32 i32 i32 i32 i32) (result i32)
    block  ;; label = @1
      local.get 2
      i32.const 1114112
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      local.get 2
      local.get 1
      i32.load offset=16
      call_indirect (type 4)
      i32.eqz
      br_if 0 (;@1;)
      i32.const 1
      return
    end
    block  ;; label = @1
      local.get 3
      br_if 0 (;@1;)
      i32.const 0
      return
    end
    local.get 0
    local.get 3
    local.get 4
    local.get 1
    i32.load offset=12
    call_indirect (type 6))
  (func $_ZN4core3fmt9Formatter9write_str17ha951e874492915b9E (type 6) (param i32 i32 i32) (result i32)
    local.get 0
    i32.load offset=20
    local.get 1
    local.get 2
    local.get 0
    i32.load offset=24
    i32.load offset=12
    call_indirect (type 6))
  (func $_ZN4core3fmt9Formatter12debug_struct17he54794dbb5a1813cE (type 12) (param i32 i32 i32 i32)
    local.get 1
    i32.load offset=20
    local.get 2
    local.get 3
    local.get 1
    i32.load offset=24
    i32.load offset=12
    call_indirect (type 6)
    local.set 3
    local.get 0
    i32.const 0
    i32.store8 offset=5
    local.get 0
    local.get 3
    i32.store8 offset=4
    local.get 0
    local.get 1
    i32.store)
  (func $_ZN4core3fmt9Formatter26debug_struct_field1_finish17ha7586c40638b50f9E (type 19) (param i32 i32 i32 i32 i32 i32 i32) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 7
    global.set $__stack_pointer
    local.get 0
    i32.load offset=20
    local.get 1
    local.get 2
    local.get 0
    i32.load offset=24
    i32.load offset=12
    call_indirect (type 6)
    local.set 2
    local.get 7
    i32.const 0
    i32.store8 offset=13
    local.get 7
    local.get 2
    i32.store8 offset=12
    local.get 7
    local.get 0
    i32.store offset=8
    local.get 7
    i32.const 8
    i32.add
    local.get 3
    local.get 4
    local.get 5
    local.get 6
    call $_ZN4core3fmt8builders11DebugStruct5field17h30cdc1b516ecb392E
    local.set 6
    local.get 7
    i32.load8_u offset=13
    local.tee 2
    local.get 7
    i32.load8_u offset=12
    local.tee 1
    i32.or
    local.set 0
    block  ;; label = @1
      local.get 2
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 1
      i32.const 1
      i32.and
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 6
        i32.load
        local.tee 0
        i32.load8_u offset=28
        i32.const 4
        i32.and
        br_if 0 (;@2;)
        local.get 0
        i32.load offset=20
        i32.const 1055863
        i32.const 2
        local.get 0
        i32.load offset=24
        i32.load offset=12
        call_indirect (type 6)
        local.set 0
        br 1 (;@1;)
      end
      local.get 0
      i32.load offset=20
      i32.const 1055862
      i32.const 1
      local.get 0
      i32.load offset=24
      i32.load offset=12
      call_indirect (type 6)
      local.set 0
    end
    local.get 7
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 0
    i32.const 1
    i32.and)
  (func $_ZN4core3fmt9Formatter26debug_struct_field2_finish17ha199c8f1cad09d1bE (type 20) (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 11
    global.set $__stack_pointer
    local.get 0
    i32.load offset=20
    local.get 1
    local.get 2
    local.get 0
    i32.load offset=24
    i32.load offset=12
    call_indirect (type 6)
    local.set 2
    local.get 11
    i32.const 0
    i32.store8 offset=13
    local.get 11
    local.get 2
    i32.store8 offset=12
    local.get 11
    local.get 0
    i32.store offset=8
    local.get 11
    i32.const 8
    i32.add
    local.get 3
    local.get 4
    local.get 5
    local.get 6
    call $_ZN4core3fmt8builders11DebugStruct5field17h30cdc1b516ecb392E
    local.get 7
    local.get 8
    local.get 9
    local.get 10
    call $_ZN4core3fmt8builders11DebugStruct5field17h30cdc1b516ecb392E
    local.set 10
    local.get 11
    i32.load8_u offset=13
    local.tee 2
    local.get 11
    i32.load8_u offset=12
    local.tee 1
    i32.or
    local.set 0
    block  ;; label = @1
      local.get 2
      i32.const 1
      i32.ne
      br_if 0 (;@1;)
      local.get 1
      i32.const 1
      i32.and
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 10
        i32.load
        local.tee 0
        i32.load8_u offset=28
        i32.const 4
        i32.and
        br_if 0 (;@2;)
        local.get 0
        i32.load offset=20
        i32.const 1055863
        i32.const 2
        local.get 0
        i32.load offset=24
        i32.load offset=12
        call_indirect (type 6)
        local.set 0
        br 1 (;@1;)
      end
      local.get 0
      i32.load offset=20
      i32.const 1055862
      i32.const 1
      local.get 0
      i32.load offset=24
      i32.load offset=12
      call_indirect (type 6)
      local.set 0
    end
    local.get 11
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 0
    i32.const 1
    i32.and)
  (func $_ZN4core3fmt9Formatter11debug_tuple17h8e8348afcbac700aE (type 12) (param i32 i32 i32 i32)
    local.get 0
    local.get 1
    i32.load offset=20
    local.get 2
    local.get 3
    local.get 1
    i32.load offset=24
    i32.load offset=12
    call_indirect (type 6)
    i32.store8 offset=8
    local.get 0
    local.get 1
    i32.store offset=4
    local.get 0
    local.get 3
    i32.eqz
    i32.store8 offset=9
    local.get 0
    i32.const 0
    i32.store)
  (func $_ZN4core3fmt9Formatter25debug_tuple_field1_finish17hcdf3e8d519221f47E (type 13) (param i32 i32 i32 i32 i32) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 5
    global.set $__stack_pointer
    local.get 5
    local.get 0
    i32.load offset=20
    local.get 1
    local.get 2
    local.get 0
    i32.load offset=24
    i32.load offset=12
    call_indirect (type 6)
    i32.store8 offset=12
    local.get 5
    local.get 0
    i32.store offset=8
    local.get 5
    local.get 2
    i32.eqz
    i32.store8 offset=13
    local.get 5
    i32.const 0
    i32.store offset=4
    local.get 5
    i32.const 4
    i32.add
    local.get 3
    local.get 4
    call $_ZN4core3fmt8builders10DebugTuple5field17h16ef0971d382d024E
    i32.load
    local.tee 2
    i32.const 0
    i32.ne
    local.get 5
    i32.load8_u offset=12
    local.tee 1
    i32.or
    local.set 0
    block  ;; label = @1
      local.get 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.const 1
      i32.and
      br_if 0 (;@1;)
      block  ;; label = @2
        block  ;; label = @3
          local.get 2
          i32.const 1
          i32.eq
          br_if 0 (;@3;)
          local.get 5
          i32.load offset=8
          local.set 2
          br 1 (;@2;)
        end
        local.get 5
        i32.load offset=8
        local.set 2
        local.get 5
        i32.load8_u offset=13
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        i32.load8_u offset=28
        i32.const 4
        i32.and
        br_if 0 (;@2;)
        i32.const 1
        local.set 0
        local.get 2
        i32.load offset=20
        i32.const 1055868
        i32.const 1
        local.get 2
        i32.load offset=24
        i32.load offset=12
        call_indirect (type 6)
        br_if 1 (;@1;)
      end
      local.get 2
      i32.load offset=20
      i32.const 1055488
      i32.const 1
      local.get 2
      i32.load offset=24
      i32.load offset=12
      call_indirect (type 6)
      local.set 0
    end
    local.get 5
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 0
    i32.const 1
    i32.and)
  (func $_ZN4core3fmt9Formatter25debug_tuple_field2_finish17h556ea0cf148a9372E (type 19) (param i32 i32 i32 i32 i32 i32 i32) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 7
    global.set $__stack_pointer
    local.get 7
    local.get 0
    i32.load offset=20
    local.get 1
    local.get 2
    local.get 0
    i32.load offset=24
    i32.load offset=12
    call_indirect (type 6)
    i32.store8 offset=12
    local.get 7
    local.get 0
    i32.store offset=8
    local.get 7
    local.get 2
    i32.eqz
    i32.store8 offset=13
    local.get 7
    i32.const 0
    i32.store offset=4
    local.get 7
    i32.const 4
    i32.add
    local.get 3
    local.get 4
    call $_ZN4core3fmt8builders10DebugTuple5field17h16ef0971d382d024E
    local.get 5
    local.get 6
    call $_ZN4core3fmt8builders10DebugTuple5field17h16ef0971d382d024E
    i32.load
    local.tee 2
    i32.const 0
    i32.ne
    local.get 7
    i32.load8_u offset=12
    local.tee 1
    i32.or
    local.set 0
    block  ;; label = @1
      local.get 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.const 1
      i32.and
      br_if 0 (;@1;)
      block  ;; label = @2
        block  ;; label = @3
          local.get 2
          i32.const 1
          i32.eq
          br_if 0 (;@3;)
          local.get 7
          i32.load offset=8
          local.set 2
          br 1 (;@2;)
        end
        local.get 7
        i32.load offset=8
        local.set 2
        local.get 7
        i32.load8_u offset=13
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        i32.load8_u offset=28
        i32.const 4
        i32.and
        br_if 0 (;@2;)
        i32.const 1
        local.set 0
        local.get 2
        i32.load offset=20
        i32.const 1055868
        i32.const 1
        local.get 2
        i32.load offset=24
        i32.load offset=12
        call_indirect (type 6)
        br_if 1 (;@1;)
      end
      local.get 2
      i32.load offset=20
      i32.const 1055488
      i32.const 1
      local.get 2
      i32.load offset=24
      i32.load offset=12
      call_indirect (type 6)
      local.set 0
    end
    local.get 7
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 0
    i32.const 1
    i32.and)
  (func $_ZN4core3fmt9Formatter10debug_list17h4a77cc254546a1a7E (type 2) (param i32 i32)
    (local i32)
    local.get 1
    i32.load offset=20
    i32.const 1055548
    i32.const 1
    local.get 1
    i32.load offset=24
    i32.load offset=12
    call_indirect (type 6)
    local.set 2
    local.get 0
    i32.const 0
    i32.store8 offset=5
    local.get 0
    local.get 2
    i32.store8 offset=4
    local.get 0
    local.get 1
    i32.store)
  (func $_ZN43_$LT$bool$u20$as$u20$core..fmt..Display$GT$3fmt17h90c5085ed9b38d31E (type 4) (param i32 i32) (result i32)
    block  ;; label = @1
      local.get 0
      i32.load8_u
      br_if 0 (;@1;)
      local.get 1
      i32.const 1056092
      i32.const 5
      call $_ZN4core3fmt9Formatter3pad17h869c031e37b3eedeE
      return
    end
    local.get 1
    i32.const 1056097
    i32.const 4
    call $_ZN4core3fmt9Formatter3pad17h869c031e37b3eedeE)
  (func $_ZN40_$LT$str$u20$as$u20$core..fmt..Debug$GT$3fmt17hd254edd9c8a21e04E (type 6) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    i32.const 1
    local.set 4
    block  ;; label = @1
      local.get 2
      i32.load offset=20
      local.tee 5
      i32.const 34
      local.get 2
      i32.load offset=24
      local.tee 6
      i32.load offset=16
      local.tee 7
      call_indirect (type 4)
      br_if 0 (;@1;)
      block  ;; label = @2
        block  ;; label = @3
          local.get 1
          br_if 0 (;@3;)
          i32.const 0
          local.set 2
          i32.const 0
          local.set 8
          br 1 (;@2;)
        end
        i32.const 0
        local.set 9
        i32.const 0
        local.get 1
        i32.sub
        local.set 10
        i32.const 0
        local.set 11
        local.get 0
        local.set 12
        local.get 1
        local.set 13
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              loop  ;; label = @6
                local.get 12
                local.get 13
                i32.add
                local.set 14
                i32.const 0
                local.set 2
                block  ;; label = @7
                  loop  ;; label = @8
                    local.get 12
                    local.get 2
                    i32.add
                    local.tee 15
                    i32.load8_u
                    local.tee 8
                    i32.const -127
                    i32.add
                    i32.const 255
                    i32.and
                    i32.const 161
                    i32.lt_u
                    br_if 1 (;@7;)
                    local.get 8
                    i32.const 34
                    i32.eq
                    br_if 1 (;@7;)
                    local.get 8
                    i32.const 92
                    i32.eq
                    br_if 1 (;@7;)
                    local.get 13
                    local.get 2
                    i32.const 1
                    i32.add
                    local.tee 2
                    i32.ne
                    br_if 0 (;@8;)
                  end
                  local.get 11
                  local.get 13
                  i32.add
                  local.set 2
                  br 4 (;@3;)
                end
                local.get 15
                i32.const 1
                i32.add
                local.set 12
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 15
                    i32.load8_s
                    local.tee 8
                    i32.const -1
                    i32.le_s
                    br_if 0 (;@8;)
                    local.get 8
                    i32.const 255
                    i32.and
                    local.set 8
                    br 1 (;@7;)
                  end
                  local.get 12
                  i32.load8_u
                  i32.const 63
                  i32.and
                  local.set 13
                  local.get 8
                  i32.const 31
                  i32.and
                  local.set 16
                  local.get 15
                  i32.const 2
                  i32.add
                  local.set 12
                  block  ;; label = @8
                    local.get 8
                    i32.const -33
                    i32.gt_u
                    br_if 0 (;@8;)
                    local.get 16
                    i32.const 6
                    i32.shl
                    local.get 13
                    i32.or
                    local.set 8
                    br 1 (;@7;)
                  end
                  local.get 13
                  i32.const 6
                  i32.shl
                  local.get 12
                  i32.load8_u
                  i32.const 63
                  i32.and
                  i32.or
                  local.set 13
                  local.get 15
                  i32.const 3
                  i32.add
                  local.set 12
                  block  ;; label = @8
                    local.get 8
                    i32.const -16
                    i32.ge_u
                    br_if 0 (;@8;)
                    local.get 13
                    local.get 16
                    i32.const 12
                    i32.shl
                    i32.or
                    local.set 8
                    br 1 (;@7;)
                  end
                  local.get 13
                  i32.const 6
                  i32.shl
                  local.get 12
                  i32.load8_u
                  i32.const 63
                  i32.and
                  i32.or
                  local.get 16
                  i32.const 18
                  i32.shl
                  i32.const 1835008
                  i32.and
                  i32.or
                  local.set 8
                  local.get 15
                  i32.const 4
                  i32.add
                  local.set 12
                end
                local.get 3
                i32.const 4
                i32.add
                local.get 8
                i32.const 65537
                call $_ZN4core4char7methods22_$LT$impl$u20$char$GT$16escape_debug_ext17h5376b048d63917f7E
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 3
                    i32.load8_u offset=4
                    i32.const 128
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 3
                    i32.load8_u offset=15
                    local.get 3
                    i32.load8_u offset=14
                    i32.sub
                    i32.const 255
                    i32.and
                    i32.const 1
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    local.get 11
                    local.get 2
                    i32.add
                    local.tee 15
                    i32.gt_u
                    br_if 1 (;@7;)
                    block  ;; label = @9
                      local.get 9
                      i32.eqz
                      br_if 0 (;@9;)
                      block  ;; label = @10
                        local.get 9
                        local.get 1
                        i32.lt_u
                        br_if 0 (;@10;)
                        local.get 9
                        local.get 1
                        i32.ne
                        br_if 3 (;@7;)
                        br 1 (;@9;)
                      end
                      local.get 0
                      local.get 9
                      i32.add
                      i32.load8_s
                      i32.const -65
                      i32.le_s
                      br_if 2 (;@7;)
                    end
                    block  ;; label = @9
                      local.get 15
                      i32.eqz
                      br_if 0 (;@9;)
                      block  ;; label = @10
                        local.get 15
                        local.get 1
                        i32.lt_u
                        br_if 0 (;@10;)
                        local.get 15
                        local.get 10
                        i32.add
                        i32.eqz
                        br_if 1 (;@9;)
                        br 3 (;@7;)
                      end
                      local.get 0
                      local.get 11
                      i32.add
                      local.get 2
                      i32.add
                      i32.load8_s
                      i32.const -65
                      i32.le_s
                      br_if 2 (;@7;)
                    end
                    local.get 5
                    local.get 0
                    local.get 9
                    i32.add
                    local.get 11
                    local.get 9
                    i32.sub
                    local.get 2
                    i32.add
                    local.get 6
                    i32.load offset=12
                    local.tee 15
                    call_indirect (type 6)
                    br_if 3 (;@5;)
                    block  ;; label = @9
                      block  ;; label = @10
                        local.get 3
                        i32.load8_u offset=4
                        i32.const 128
                        i32.ne
                        br_if 0 (;@10;)
                        local.get 5
                        local.get 3
                        i32.load offset=8
                        local.get 7
                        call_indirect (type 4)
                        i32.eqz
                        br_if 1 (;@9;)
                        br 5 (;@5;)
                      end
                      local.get 5
                      local.get 3
                      i32.const 4
                      i32.add
                      local.get 3
                      i32.load8_u offset=14
                      local.tee 13
                      i32.add
                      local.get 3
                      i32.load8_u offset=15
                      local.get 13
                      i32.sub
                      local.get 15
                      call_indirect (type 6)
                      br_if 4 (;@5;)
                    end
                    block  ;; label = @9
                      block  ;; label = @10
                        local.get 8
                        i32.const 128
                        i32.ge_u
                        br_if 0 (;@10;)
                        i32.const 1
                        local.set 15
                        br 1 (;@9;)
                      end
                      block  ;; label = @10
                        local.get 8
                        i32.const 2048
                        i32.ge_u
                        br_if 0 (;@10;)
                        i32.const 2
                        local.set 15
                        br 1 (;@9;)
                      end
                      i32.const 3
                      i32.const 4
                      local.get 8
                      i32.const 65536
                      i32.lt_u
                      select
                      local.set 15
                    end
                    local.get 15
                    local.get 11
                    i32.add
                    local.get 2
                    i32.add
                    local.set 9
                  end
                  block  ;; label = @8
                    block  ;; label = @9
                      local.get 8
                      i32.const 128
                      i32.ge_u
                      br_if 0 (;@9;)
                      i32.const 1
                      local.set 8
                      br 1 (;@8;)
                    end
                    block  ;; label = @9
                      local.get 8
                      i32.const 2048
                      i32.ge_u
                      br_if 0 (;@9;)
                      i32.const 2
                      local.set 8
                      br 1 (;@8;)
                    end
                    i32.const 3
                    i32.const 4
                    local.get 8
                    i32.const 65536
                    i32.lt_u
                    select
                    local.set 8
                  end
                  local.get 8
                  local.get 11
                  i32.add
                  local.tee 8
                  local.get 2
                  i32.add
                  local.set 11
                  local.get 14
                  local.get 12
                  i32.sub
                  local.tee 13
                  i32.eqz
                  br_if 3 (;@4;)
                  br 1 (;@6;)
                end
              end
              local.get 0
              local.get 1
              local.get 9
              local.get 15
              i32.const 1056104
              call $_ZN4core3str16slice_error_fail17ha36806ef39029680E
              unreachable
            end
            i32.const 1
            local.set 4
            br 3 (;@1;)
          end
          local.get 8
          local.get 2
          i32.add
          local.set 2
        end
        block  ;; label = @3
          local.get 9
          local.get 2
          i32.gt_u
          br_if 0 (;@3;)
          i32.const 0
          local.set 8
          block  ;; label = @4
            local.get 9
            i32.eqz
            br_if 0 (;@4;)
            block  ;; label = @5
              local.get 9
              local.get 1
              i32.lt_u
              br_if 0 (;@5;)
              local.get 9
              local.set 8
              local.get 9
              local.get 1
              i32.ne
              br_if 2 (;@3;)
              br 1 (;@4;)
            end
            local.get 9
            local.set 8
            local.get 0
            local.get 9
            i32.add
            i32.load8_s
            i32.const -65
            i32.le_s
            br_if 1 (;@3;)
          end
          block  ;; label = @4
            local.get 2
            br_if 0 (;@4;)
            i32.const 0
            local.set 2
            br 2 (;@2;)
          end
          block  ;; label = @4
            local.get 2
            local.get 1
            i32.lt_u
            br_if 0 (;@4;)
            local.get 8
            local.set 9
            local.get 2
            local.get 1
            i32.eq
            br_if 2 (;@2;)
            br 1 (;@3;)
          end
          local.get 8
          local.set 9
          local.get 0
          local.get 2
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          br_if 1 (;@2;)
        end
        local.get 0
        local.get 1
        local.get 9
        local.get 2
        i32.const 1056120
        call $_ZN4core3str16slice_error_fail17ha36806ef39029680E
        unreachable
      end
      local.get 5
      local.get 0
      local.get 8
      i32.add
      local.get 2
      local.get 8
      i32.sub
      local.get 6
      i32.load offset=12
      call_indirect (type 6)
      br_if 0 (;@1;)
      local.get 5
      i32.const 34
      local.get 7
      call_indirect (type 4)
      local.set 4
    end
    local.get 3
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 4)
  (func $_ZN4core3str16slice_error_fail17ha36806ef39029680E (type 7) (param i32 i32 i32 i32 i32)
    local.get 0
    local.get 1
    local.get 2
    local.get 3
    local.get 4
    call $_ZN4core3str19slice_error_fail_rt17h55ab681dd9553eeaE
    unreachable)
  (func $_ZN41_$LT$char$u20$as$u20$core..fmt..Debug$GT$3fmt17h6ec13b7460ecb1beE (type 4) (param i32 i32) (result i32)
    (local i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    i32.const 1
    local.set 3
    block  ;; label = @1
      local.get 1
      i32.load offset=20
      local.tee 4
      i32.const 39
      local.get 1
      i32.load offset=24
      local.tee 5
      i32.load offset=16
      local.tee 1
      call_indirect (type 4)
      br_if 0 (;@1;)
      local.get 2
      i32.const 4
      i32.add
      local.get 0
      i32.load
      i32.const 257
      call $_ZN4core4char7methods22_$LT$impl$u20$char$GT$16escape_debug_ext17h5376b048d63917f7E
      block  ;; label = @2
        block  ;; label = @3
          local.get 2
          i32.load8_u offset=4
          i32.const 128
          i32.ne
          br_if 0 (;@3;)
          local.get 4
          local.get 2
          i32.load offset=8
          local.get 1
          call_indirect (type 4)
          i32.eqz
          br_if 1 (;@2;)
          br 2 (;@1;)
        end
        local.get 4
        local.get 2
        i32.const 4
        i32.add
        local.get 2
        i32.load8_u offset=14
        local.tee 0
        i32.add
        local.get 2
        i32.load8_u offset=15
        local.get 0
        i32.sub
        local.get 5
        i32.load offset=12
        call_indirect (type 6)
        br_if 1 (;@1;)
      end
      local.get 4
      i32.const 39
      local.get 1
      call_indirect (type 4)
      local.set 3
    end
    local.get 2
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 3)
  (func $_ZN4core5slice6memchr14memchr_aligned17hc466838cdf21c242E (type 12) (param i32 i32 i32 i32)
    (local i32 i32 i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 2
            i32.const 3
            i32.add
            i32.const -4
            i32.and
            local.tee 4
            local.get 2
            i32.eq
            br_if 0 (;@4;)
            local.get 4
            local.get 2
            i32.sub
            local.tee 4
            local.get 3
            local.get 4
            local.get 3
            i32.lt_u
            select
            local.tee 4
            i32.eqz
            br_if 0 (;@4;)
            i32.const 0
            local.set 5
            local.get 1
            i32.const 255
            i32.and
            local.set 6
            i32.const 1
            local.set 7
            loop  ;; label = @5
              local.get 2
              local.get 5
              i32.add
              i32.load8_u
              local.get 6
              i32.eq
              br_if 4 (;@1;)
              local.get 4
              local.get 5
              i32.const 1
              i32.add
              local.tee 5
              i32.ne
              br_if 0 (;@5;)
            end
            local.get 4
            local.get 3
            i32.const -8
            i32.add
            local.tee 8
            i32.gt_u
            br_if 2 (;@2;)
            br 1 (;@3;)
          end
          local.get 3
          i32.const -8
          i32.add
          local.set 8
          i32.const 0
          local.set 4
        end
        local.get 1
        i32.const 255
        i32.and
        i32.const 16843009
        i32.mul
        local.set 5
        loop  ;; label = @3
          i32.const 16843008
          local.get 2
          local.get 4
          i32.add
          local.tee 6
          i32.load
          local.get 5
          i32.xor
          local.tee 7
          i32.sub
          local.get 7
          i32.or
          i32.const 16843008
          local.get 6
          i32.const 4
          i32.add
          i32.load
          local.get 5
          i32.xor
          local.tee 6
          i32.sub
          local.get 6
          i32.or
          i32.and
          i32.const -2139062144
          i32.and
          i32.const -2139062144
          i32.ne
          br_if 1 (;@2;)
          local.get 4
          i32.const 8
          i32.add
          local.tee 4
          local.get 8
          i32.le_u
          br_if 0 (;@3;)
        end
      end
      block  ;; label = @2
        local.get 4
        local.get 3
        i32.eq
        br_if 0 (;@2;)
        local.get 1
        i32.const 255
        i32.and
        local.set 5
        i32.const 1
        local.set 7
        loop  ;; label = @3
          block  ;; label = @4
            local.get 2
            local.get 4
            i32.add
            i32.load8_u
            local.get 5
            i32.ne
            br_if 0 (;@4;)
            local.get 4
            local.set 5
            br 3 (;@1;)
          end
          local.get 3
          local.get 4
          i32.const 1
          i32.add
          local.tee 4
          i32.ne
          br_if 0 (;@3;)
        end
      end
      i32.const 0
      local.set 7
    end
    local.get 0
    local.get 5
    i32.store offset=4
    local.get 0
    local.get 7
    i32.store)
  (func $_ZN4core5slice5index24slice_end_index_len_fail8do_panic7runtime17h40abf6316be1d38dE (type 3) (param i32 i32 i32)
    (local i32 i64)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 3
    local.get 1
    i32.store offset=4
    local.get 3
    local.get 0
    i32.store
    local.get 3
    i32.const 2
    i32.store offset=12
    local.get 3
    i32.const 1058028
    i32.store offset=8
    local.get 3
    i64.const 2
    i64.store offset=20 align=4
    local.get 3
    i32.const 22
    i64.extend_i32_u
    i64.const 32
    i64.shl
    local.tee 4
    local.get 3
    i32.const 4
    i32.add
    i64.extend_i32_u
    i64.or
    i64.store offset=40
    local.get 3
    local.get 4
    local.get 3
    i64.extend_i32_u
    i64.or
    i64.store offset=32
    local.get 3
    local.get 3
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 3
    i32.const 8
    i32.add
    local.get 2
    call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
    unreachable)
  (func $_ZN4core5slice5index22slice_index_order_fail8do_panic7runtime17h3eb440503b756a77E (type 3) (param i32 i32 i32)
    (local i32 i64)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 3
    local.get 1
    i32.store offset=4
    local.get 3
    local.get 0
    i32.store
    local.get 3
    i32.const 2
    i32.store offset=12
    local.get 3
    i32.const 1058080
    i32.store offset=8
    local.get 3
    i64.const 2
    i64.store offset=20 align=4
    local.get 3
    i32.const 22
    i64.extend_i32_u
    i64.const 32
    i64.shl
    local.tee 4
    local.get 3
    i32.const 4
    i32.add
    i64.extend_i32_u
    i64.or
    i64.store offset=40
    local.get 3
    local.get 4
    local.get 3
    i64.extend_i32_u
    i64.or
    i64.store offset=32
    local.get 3
    local.get 3
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 3
    i32.const 8
    i32.add
    local.get 2
    call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
    unreachable)
  (func $_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$i8$GT$3fmt17h7609c3b52c3b7d69E (type 4) (param i32 i32) (result i32)
    (local i32 i32 i32)
    global.get $__stack_pointer
    i32.const 128
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 0
    i32.load8_u
    local.set 3
    i32.const 0
    local.set 0
    loop  ;; label = @1
      local.get 2
      local.get 0
      i32.add
      i32.const 127
      i32.add
      local.get 3
      i32.const 15
      i32.and
      local.tee 4
      i32.const 48
      i32.or
      local.get 4
      i32.const 55
      i32.add
      local.get 4
      i32.const 10
      i32.lt_u
      select
      i32.store8
      local.get 0
      i32.const -1
      i32.add
      local.set 0
      local.get 3
      i32.const 255
      i32.and
      local.tee 4
      i32.const 4
      i32.shr_u
      local.set 3
      local.get 4
      i32.const 15
      i32.gt_u
      br_if 0 (;@1;)
    end
    local.get 1
    i32.const 1
    i32.const 1055871
    i32.const 2
    local.get 2
    local.get 0
    i32.add
    i32.const 128
    i32.add
    i32.const 0
    local.get 0
    i32.sub
    call $_ZN4core3fmt9Formatter12pad_integral17h121008db9b1c3bbbE
    local.set 0
    local.get 2
    i32.const 128
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN4core3str19slice_error_fail_rt17h55ab681dd9553eeaE (type 7) (param i32 i32 i32 i32 i32)
    (local i32 i32 i32 i32 i32 i64)
    global.get $__stack_pointer
    i32.const 112
    i32.sub
    local.tee 5
    global.set $__stack_pointer
    local.get 5
    local.get 3
    i32.store offset=12
    local.get 5
    local.get 2
    i32.store offset=8
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      local.get 1
                      i32.const 257
                      i32.lt_u
                      br_if 0 (;@9;)
                      block  ;; label = @10
                        local.get 0
                        i32.load8_s offset=256
                        i32.const -65
                        i32.le_s
                        br_if 0 (;@10;)
                        i32.const 3
                        local.set 6
                        br 3 (;@7;)
                      end
                      local.get 0
                      i32.load8_s offset=255
                      i32.const -65
                      i32.le_s
                      br_if 1 (;@8;)
                      i32.const 2
                      local.set 6
                      br 2 (;@7;)
                    end
                    local.get 5
                    local.get 1
                    i32.store offset=20
                    local.get 5
                    local.get 0
                    i32.store offset=16
                    i32.const 0
                    local.set 6
                    i32.const 1
                    local.set 7
                    br 2 (;@6;)
                  end
                  local.get 0
                  i32.load8_s offset=254
                  i32.const -65
                  i32.gt_s
                  local.set 6
                end
                local.get 0
                local.get 6
                i32.const 253
                i32.add
                local.tee 6
                i32.add
                i32.load8_s
                i32.const -65
                i32.le_s
                br_if 1 (;@5;)
                local.get 5
                local.get 6
                i32.store offset=20
                local.get 5
                local.get 0
                i32.store offset=16
                i32.const 5
                local.set 6
                i32.const 1056155
                local.set 7
              end
              local.get 5
              local.get 6
              i32.store offset=28
              local.get 5
              local.get 7
              i32.store offset=24
              block  ;; label = @6
                local.get 2
                local.get 1
                i32.gt_u
                local.tee 6
                br_if 0 (;@6;)
                local.get 3
                local.get 1
                i32.gt_u
                br_if 0 (;@6;)
                local.get 2
                local.get 3
                i32.gt_u
                br_if 2 (;@4;)
                block  ;; label = @7
                  local.get 2
                  i32.eqz
                  br_if 0 (;@7;)
                  local.get 2
                  local.get 1
                  i32.ge_u
                  br_if 0 (;@7;)
                  local.get 5
                  i32.const 12
                  i32.add
                  local.get 5
                  i32.const 8
                  i32.add
                  local.get 0
                  local.get 2
                  i32.add
                  i32.load8_s
                  i32.const -65
                  i32.gt_s
                  select
                  i32.load
                  local.set 3
                end
                local.get 5
                local.get 3
                i32.store offset=32
                local.get 1
                local.set 2
                block  ;; label = @7
                  local.get 3
                  local.get 1
                  i32.ge_u
                  br_if 0 (;@7;)
                  local.get 3
                  i32.const 1
                  i32.add
                  local.tee 7
                  i32.const 0
                  local.get 3
                  i32.const -3
                  i32.add
                  local.tee 2
                  local.get 2
                  local.get 3
                  i32.gt_u
                  select
                  local.tee 2
                  i32.lt_u
                  br_if 4 (;@3;)
                  block  ;; label = @8
                    local.get 7
                    local.get 2
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 7
                    local.get 2
                    i32.sub
                    local.set 8
                    block  ;; label = @9
                      local.get 0
                      local.get 3
                      i32.add
                      i32.load8_s
                      i32.const -65
                      i32.le_s
                      br_if 0 (;@9;)
                      local.get 8
                      i32.const -1
                      i32.add
                      local.set 6
                      br 1 (;@8;)
                    end
                    local.get 2
                    local.get 3
                    i32.eq
                    br_if 0 (;@8;)
                    block  ;; label = @9
                      local.get 0
                      local.get 7
                      i32.add
                      local.tee 7
                      i32.const -2
                      i32.add
                      local.tee 3
                      i32.load8_s
                      i32.const -65
                      i32.le_s
                      br_if 0 (;@9;)
                      local.get 8
                      i32.const -2
                      i32.add
                      local.set 6
                      br 1 (;@8;)
                    end
                    local.get 0
                    local.get 2
                    i32.add
                    local.tee 9
                    local.get 3
                    i32.eq
                    br_if 0 (;@8;)
                    block  ;; label = @9
                      local.get 7
                      i32.const -3
                      i32.add
                      local.tee 3
                      i32.load8_s
                      i32.const -65
                      i32.le_s
                      br_if 0 (;@9;)
                      local.get 8
                      i32.const -3
                      i32.add
                      local.set 6
                      br 1 (;@8;)
                    end
                    local.get 9
                    local.get 3
                    i32.eq
                    br_if 0 (;@8;)
                    block  ;; label = @9
                      local.get 7
                      i32.const -4
                      i32.add
                      local.tee 3
                      i32.load8_s
                      i32.const -65
                      i32.le_s
                      br_if 0 (;@9;)
                      local.get 8
                      i32.const -4
                      i32.add
                      local.set 6
                      br 1 (;@8;)
                    end
                    local.get 9
                    local.get 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 8
                    i32.const -5
                    i32.add
                    local.set 6
                  end
                  local.get 6
                  local.get 2
                  i32.add
                  local.set 2
                end
                block  ;; label = @7
                  local.get 2
                  i32.eqz
                  br_if 0 (;@7;)
                  block  ;; label = @8
                    local.get 2
                    local.get 1
                    i32.lt_u
                    br_if 0 (;@8;)
                    local.get 2
                    local.get 1
                    i32.eq
                    br_if 1 (;@7;)
                    br 7 (;@1;)
                  end
                  local.get 0
                  local.get 2
                  i32.add
                  i32.load8_s
                  i32.const -65
                  i32.le_s
                  br_if 6 (;@1;)
                end
                local.get 2
                local.get 1
                i32.eq
                br_if 4 (;@2;)
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      block  ;; label = @10
                        local.get 0
                        local.get 2
                        i32.add
                        local.tee 3
                        i32.load8_s
                        local.tee 1
                        i32.const -1
                        i32.gt_s
                        br_if 0 (;@10;)
                        local.get 3
                        i32.load8_u offset=1
                        i32.const 63
                        i32.and
                        local.set 0
                        local.get 1
                        i32.const 31
                        i32.and
                        local.set 6
                        local.get 1
                        i32.const -33
                        i32.gt_u
                        br_if 1 (;@9;)
                        local.get 6
                        i32.const 6
                        i32.shl
                        local.get 0
                        i32.or
                        local.set 1
                        br 2 (;@8;)
                      end
                      local.get 5
                      local.get 1
                      i32.const 255
                      i32.and
                      i32.store offset=36
                      i32.const 1
                      local.set 1
                      br 2 (;@7;)
                    end
                    local.get 0
                    i32.const 6
                    i32.shl
                    local.get 3
                    i32.load8_u offset=2
                    i32.const 63
                    i32.and
                    i32.or
                    local.set 0
                    block  ;; label = @9
                      local.get 1
                      i32.const -16
                      i32.ge_u
                      br_if 0 (;@9;)
                      local.get 0
                      local.get 6
                      i32.const 12
                      i32.shl
                      i32.or
                      local.set 1
                      br 1 (;@8;)
                    end
                    local.get 0
                    i32.const 6
                    i32.shl
                    local.get 3
                    i32.load8_u offset=3
                    i32.const 63
                    i32.and
                    i32.or
                    local.get 6
                    i32.const 18
                    i32.shl
                    i32.const 1835008
                    i32.and
                    i32.or
                    local.tee 1
                    i32.const 1114112
                    i32.eq
                    br_if 6 (;@2;)
                  end
                  local.get 5
                  local.get 1
                  i32.store offset=36
                  block  ;; label = @8
                    local.get 1
                    i32.const 128
                    i32.ge_u
                    br_if 0 (;@8;)
                    i32.const 1
                    local.set 1
                    br 1 (;@7;)
                  end
                  block  ;; label = @8
                    local.get 1
                    i32.const 2048
                    i32.ge_u
                    br_if 0 (;@8;)
                    i32.const 2
                    local.set 1
                    br 1 (;@7;)
                  end
                  i32.const 3
                  i32.const 4
                  local.get 1
                  i32.const 65536
                  i32.lt_u
                  select
                  local.set 1
                end
                local.get 5
                local.get 2
                i32.store offset=40
                local.get 5
                local.get 1
                local.get 2
                i32.add
                i32.store offset=44
                local.get 5
                i32.const 5
                i32.store offset=52
                local.get 5
                i32.const 1056292
                i32.store offset=48
                local.get 5
                i64.const 5
                i64.store offset=60 align=4
                local.get 5
                i32.const 57
                i64.extend_i32_u
                i64.const 32
                i64.shl
                local.tee 10
                local.get 5
                i32.const 24
                i32.add
                i64.extend_i32_u
                i64.or
                i64.store offset=104
                local.get 5
                local.get 10
                local.get 5
                i32.const 16
                i32.add
                i64.extend_i32_u
                i64.or
                i64.store offset=96
                local.get 5
                i32.const 59
                i64.extend_i32_u
                i64.const 32
                i64.shl
                local.get 5
                i32.const 40
                i32.add
                i64.extend_i32_u
                i64.or
                i64.store offset=88
                local.get 5
                i32.const 60
                i64.extend_i32_u
                i64.const 32
                i64.shl
                local.get 5
                i32.const 36
                i32.add
                i64.extend_i32_u
                i64.or
                i64.store offset=80
                local.get 5
                i32.const 22
                i64.extend_i32_u
                i64.const 32
                i64.shl
                local.get 5
                i32.const 32
                i32.add
                i64.extend_i32_u
                i64.or
                i64.store offset=72
                local.get 5
                local.get 5
                i32.const 72
                i32.add
                i32.store offset=56
                local.get 5
                i32.const 48
                i32.add
                local.get 4
                call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
                unreachable
              end
              local.get 5
              local.get 2
              local.get 3
              local.get 6
              select
              i32.store offset=40
              local.get 5
              i32.const 3
              i32.store offset=52
              local.get 5
              i32.const 1056356
              i32.store offset=48
              local.get 5
              i64.const 3
              i64.store offset=60 align=4
              local.get 5
              i32.const 57
              i64.extend_i32_u
              i64.const 32
              i64.shl
              local.tee 10
              local.get 5
              i32.const 24
              i32.add
              i64.extend_i32_u
              i64.or
              i64.store offset=88
              local.get 5
              local.get 10
              local.get 5
              i32.const 16
              i32.add
              i64.extend_i32_u
              i64.or
              i64.store offset=80
              local.get 5
              i32.const 22
              i64.extend_i32_u
              i64.const 32
              i64.shl
              local.get 5
              i32.const 40
              i32.add
              i64.extend_i32_u
              i64.or
              i64.store offset=72
              local.get 5
              local.get 5
              i32.const 72
              i32.add
              i32.store offset=56
              local.get 5
              i32.const 48
              i32.add
              local.get 4
              call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
              unreachable
            end
            local.get 0
            local.get 1
            i32.const 0
            local.get 6
            local.get 4
            call $_ZN4core3str16slice_error_fail17ha36806ef39029680E
            unreachable
          end
          local.get 5
          i32.const 4
          i32.store offset=52
          local.get 5
          i32.const 1056196
          i32.store offset=48
          local.get 5
          i64.const 4
          i64.store offset=60 align=4
          local.get 5
          i32.const 57
          i64.extend_i32_u
          i64.const 32
          i64.shl
          local.tee 10
          local.get 5
          i32.const 24
          i32.add
          i64.extend_i32_u
          i64.or
          i64.store offset=96
          local.get 5
          local.get 10
          local.get 5
          i32.const 16
          i32.add
          i64.extend_i32_u
          i64.or
          i64.store offset=88
          local.get 5
          i32.const 22
          i64.extend_i32_u
          i64.const 32
          i64.shl
          local.tee 10
          local.get 5
          i32.const 12
          i32.add
          i64.extend_i32_u
          i64.or
          i64.store offset=80
          local.get 5
          local.get 10
          local.get 5
          i32.const 8
          i32.add
          i64.extend_i32_u
          i64.or
          i64.store offset=72
          local.get 5
          local.get 5
          i32.const 72
          i32.add
          i32.store offset=56
          local.get 5
          i32.const 48
          i32.add
          local.get 4
          call $_ZN4core9panicking9panic_fmt17h931ec2537c26fa22E
          unreachable
        end
        local.get 2
        local.get 7
        i32.const 1056380
        call $_ZN4core5slice5index22slice_index_order_fail17hce8d8012711d5a00E
        unreachable
      end
      local.get 4
      call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
      unreachable
    end
    local.get 0
    local.get 1
    local.get 2
    local.get 1
    local.get 4
    call $_ZN4core3str16slice_error_fail17ha36806ef39029680E
    unreachable)
  (func $_ZN4core7unicode9printable5check17h54417f65c5968d49E (type 19) (param i32 i32 i32 i32 i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32)
    i32.const 1
    local.set 7
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        local.get 2
        i32.const 1
        i32.shl
        i32.add
        local.set 8
        local.get 0
        i32.const 65280
        i32.and
        i32.const 8
        i32.shr_u
        local.set 9
        i32.const 0
        local.set 10
        local.get 0
        i32.const 255
        i32.and
        local.set 11
        loop  ;; label = @3
          local.get 1
          i32.const 2
          i32.add
          local.set 12
          local.get 10
          local.get 1
          i32.load8_u offset=1
          local.tee 2
          i32.add
          local.set 13
          block  ;; label = @4
            local.get 1
            i32.load8_u
            local.tee 1
            local.get 9
            i32.eq
            br_if 0 (;@4;)
            local.get 1
            local.get 9
            i32.gt_u
            br_if 2 (;@2;)
            local.get 13
            local.set 10
            local.get 12
            local.set 1
            local.get 12
            local.get 8
            i32.eq
            br_if 2 (;@2;)
            br 1 (;@3;)
          end
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                local.get 13
                local.get 10
                i32.lt_u
                br_if 0 (;@6;)
                local.get 13
                local.get 4
                i32.gt_u
                br_if 1 (;@5;)
                local.get 3
                local.get 10
                i32.add
                local.set 1
                loop  ;; label = @7
                  local.get 2
                  i32.eqz
                  br_if 3 (;@4;)
                  local.get 2
                  i32.const -1
                  i32.add
                  local.set 2
                  local.get 1
                  i32.load8_u
                  local.set 10
                  local.get 1
                  i32.const 1
                  i32.add
                  local.set 1
                  local.get 10
                  local.get 11
                  i32.ne
                  br_if 0 (;@7;)
                end
                i32.const 0
                local.set 7
                br 5 (;@1;)
              end
              local.get 10
              local.get 13
              i32.const 1056444
              call $_ZN4core5slice5index22slice_index_order_fail17hce8d8012711d5a00E
              unreachable
            end
            local.get 13
            local.get 4
            i32.const 1056444
            call $_ZN4core5slice5index24slice_end_index_len_fail17h07937a589bfe269aE
            unreachable
          end
          local.get 13
          local.set 10
          local.get 12
          local.set 1
          local.get 12
          local.get 8
          i32.ne
          br_if 0 (;@3;)
        end
      end
      local.get 6
      i32.eqz
      br_if 0 (;@1;)
      local.get 5
      local.get 6
      i32.add
      local.set 11
      local.get 0
      i32.const 65535
      i32.and
      local.set 1
      i32.const 1
      local.set 7
      loop  ;; label = @2
        local.get 5
        i32.const 1
        i32.add
        local.set 10
        block  ;; label = @3
          block  ;; label = @4
            local.get 5
            i32.load8_s
            local.tee 2
            i32.const 0
            i32.lt_s
            br_if 0 (;@4;)
            local.get 10
            local.set 5
            br 1 (;@3;)
          end
          block  ;; label = @4
            local.get 10
            local.get 11
            i32.eq
            br_if 0 (;@4;)
            local.get 2
            i32.const 127
            i32.and
            i32.const 8
            i32.shl
            local.get 5
            i32.load8_u offset=1
            i32.or
            local.set 2
            local.get 5
            i32.const 2
            i32.add
            local.set 5
            br 1 (;@3;)
          end
          i32.const 1056428
          call $_ZN4core6option13unwrap_failed17h7534a988a872bb9fE
          unreachable
        end
        local.get 1
        local.get 2
        i32.sub
        local.tee 1
        i32.const 0
        i32.lt_s
        br_if 1 (;@1;)
        local.get 7
        i32.const 1
        i32.xor
        local.set 7
        local.get 5
        local.get 11
        i32.ne
        br_if 0 (;@2;)
      end
    end
    local.get 7
    i32.const 1
    i32.and)
  (func $_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$i8$GT$3fmt17hdee09822fcf3ff73E (type 4) (param i32 i32) (result i32)
    (local i32 i32 i32)
    global.get $__stack_pointer
    i32.const 128
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 0
    i32.load8_u
    local.set 3
    i32.const 0
    local.set 0
    loop  ;; label = @1
      local.get 2
      local.get 0
      i32.add
      i32.const 127
      i32.add
      local.get 3
      i32.const 15
      i32.and
      local.tee 4
      i32.const 48
      i32.or
      local.get 4
      i32.const 87
      i32.add
      local.get 4
      i32.const 10
      i32.lt_u
      select
      i32.store8
      local.get 0
      i32.const -1
      i32.add
      local.set 0
      local.get 3
      i32.const 255
      i32.and
      local.tee 4
      i32.const 4
      i32.shr_u
      local.set 3
      local.get 4
      i32.const 15
      i32.gt_u
      br_if 0 (;@1;)
    end
    local.get 1
    i32.const 1
    i32.const 1055871
    i32.const 2
    local.get 2
    local.get 0
    i32.add
    i32.const 128
    i32.add
    i32.const 0
    local.get 0
    i32.sub
    call $_ZN4core3fmt9Formatter12pad_integral17h121008db9b1c3bbbE
    local.set 0
    local.get 2
    i32.const 128
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN4core3fmt3num53_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$i32$GT$3fmt17h2e92699d27a37844E (type 4) (param i32 i32) (result i32)
    (local i32 i32 i32)
    global.get $__stack_pointer
    i32.const 128
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 0
    i32.load
    local.set 0
    i32.const 0
    local.set 3
    loop  ;; label = @1
      local.get 2
      local.get 3
      i32.add
      i32.const 127
      i32.add
      local.get 0
      i32.const 15
      i32.and
      local.tee 4
      i32.const 48
      i32.or
      local.get 4
      i32.const 55
      i32.add
      local.get 4
      i32.const 10
      i32.lt_u
      select
      i32.store8
      local.get 3
      i32.const -1
      i32.add
      local.set 3
      local.get 0
      i32.const 15
      i32.gt_u
      local.set 4
      local.get 0
      i32.const 4
      i32.shr_u
      local.set 0
      local.get 4
      br_if 0 (;@1;)
    end
    local.get 1
    i32.const 1
    i32.const 1055871
    i32.const 2
    local.get 2
    local.get 3
    i32.add
    i32.const 128
    i32.add
    i32.const 0
    local.get 3
    i32.sub
    call $_ZN4core3fmt9Formatter12pad_integral17h121008db9b1c3bbbE
    local.set 0
    local.get 2
    i32.const 128
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i32$GT$3fmt17hfd770d1228523106E (type 4) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.load
        local.tee 0
        i32.const -1
        i32.gt_s
        br_if 0 (;@2;)
        i32.const 0
        local.get 0
        i32.sub
        i32.const 0
        local.get 1
        call $_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17h4f3209f6e643fb87E
        local.set 0
        br 1 (;@1;)
      end
      i32.const 10
      local.set 3
      block  ;; label = @2
        block  ;; label = @3
          local.get 0
          i32.const 10000
          i32.ge_u
          br_if 0 (;@3;)
          local.get 0
          local.set 4
          br 1 (;@2;)
        end
        i32.const 10
        local.set 3
        loop  ;; label = @3
          local.get 2
          i32.const 6
          i32.add
          local.get 3
          i32.add
          local.tee 5
          i32.const -4
          i32.add
          local.get 0
          local.get 0
          i32.const 10000
          i32.div_u
          local.tee 4
          i32.const 10000
          i32.mul
          i32.sub
          local.tee 6
          i32.const 65535
          i32.and
          i32.const 100
          i32.div_u
          local.tee 7
          i32.const 1
          i32.shl
          i32.const 1055873
          i32.add
          i32.load16_u align=1
          i32.store16 align=1
          local.get 5
          i32.const -2
          i32.add
          local.get 6
          local.get 7
          i32.const 100
          i32.mul
          i32.sub
          i32.const 65535
          i32.and
          i32.const 1
          i32.shl
          i32.const 1055873
          i32.add
          i32.load16_u align=1
          i32.store16 align=1
          local.get 3
          i32.const -4
          i32.add
          local.set 3
          local.get 0
          i32.const 99999999
          i32.gt_u
          local.set 5
          local.get 4
          local.set 0
          local.get 5
          br_if 0 (;@3;)
        end
      end
      block  ;; label = @2
        block  ;; label = @3
          local.get 4
          i32.const 99
          i32.gt_u
          br_if 0 (;@3;)
          local.get 4
          local.set 0
          br 1 (;@2;)
        end
        local.get 2
        i32.const 6
        i32.add
        local.get 3
        i32.const -2
        i32.add
        local.tee 3
        i32.add
        local.get 4
        local.get 4
        i32.const 100
        i32.div_u
        local.tee 0
        i32.const 100
        i32.mul
        i32.sub
        i32.const 1
        i32.shl
        i32.const 1055873
        i32.add
        i32.load16_u align=1
        i32.store16 align=1
      end
      block  ;; label = @2
        block  ;; label = @3
          local.get 0
          i32.const 10
          i32.lt_u
          br_if 0 (;@3;)
          local.get 2
          i32.const 6
          i32.add
          local.get 3
          i32.const -2
          i32.add
          local.tee 3
          i32.add
          local.get 0
          i32.const 1
          i32.shl
          i32.const 1055873
          i32.add
          i32.load16_u align=1
          i32.store16 align=1
          br 1 (;@2;)
        end
        local.get 2
        i32.const 6
        i32.add
        local.get 3
        i32.const -1
        i32.add
        local.tee 3
        i32.add
        local.get 0
        i32.const 48
        i32.or
        i32.store8
      end
      local.get 1
      i32.const 1
      i32.const 1
      i32.const 0
      local.get 2
      i32.const 6
      i32.add
      local.get 3
      i32.add
      i32.const 10
      local.get 3
      i32.sub
      call $_ZN4core3fmt9Formatter12pad_integral17h121008db9b1c3bbbE
      local.set 0
    end
    local.get 2
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $_ZN17compiler_builtins3mem7memmove17hd6c723cac9ad46b4E (type 6) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 0
          local.get 1
          i32.sub
          local.get 2
          i32.ge_u
          br_if 0 (;@3;)
          local.get 1
          local.get 2
          i32.add
          local.set 3
          local.get 0
          local.get 2
          i32.add
          local.set 4
          local.get 2
          i32.const 16
          i32.lt_u
          br_if 1 (;@2;)
          i32.const 0
          local.get 4
          i32.const 3
          i32.and
          local.tee 5
          i32.sub
          local.set 6
          block  ;; label = @4
            local.get 4
            i32.const -4
            i32.and
            local.tee 7
            local.get 4
            i32.ge_u
            br_if 0 (;@4;)
            local.get 5
            i32.const -1
            i32.add
            local.set 8
            block  ;; label = @5
              block  ;; label = @6
                local.get 5
                br_if 0 (;@6;)
                local.get 3
                local.set 9
                br 1 (;@5;)
              end
              local.get 5
              local.set 10
              local.get 3
              local.set 9
              loop  ;; label = @6
                local.get 4
                i32.const -1
                i32.add
                local.tee 4
                local.get 9
                i32.const -1
                i32.add
                local.tee 9
                i32.load8_u
                i32.store8
                local.get 10
                i32.const -1
                i32.add
                local.tee 10
                br_if 0 (;@6;)
              end
            end
            local.get 8
            i32.const 3
            i32.lt_u
            br_if 0 (;@4;)
            local.get 9
            i32.const -4
            i32.add
            local.set 9
            loop  ;; label = @5
              local.get 4
              i32.const -1
              i32.add
              local.get 9
              i32.const 3
              i32.add
              i32.load8_u
              i32.store8
              local.get 4
              i32.const -2
              i32.add
              local.get 9
              i32.const 2
              i32.add
              i32.load8_u
              i32.store8
              local.get 4
              i32.const -3
              i32.add
              local.get 9
              i32.const 1
              i32.add
              i32.load8_u
              i32.store8
              local.get 4
              i32.const -4
              i32.add
              local.tee 4
              local.get 9
              i32.load8_u
              i32.store8
              local.get 9
              i32.const -4
              i32.add
              local.set 9
              local.get 7
              local.get 4
              i32.lt_u
              br_if 0 (;@5;)
            end
          end
          local.get 7
          local.get 2
          local.get 5
          i32.sub
          local.tee 9
          i32.const -4
          i32.and
          local.tee 2
          i32.sub
          local.set 4
          i32.const 0
          local.get 2
          i32.sub
          local.set 10
          block  ;; label = @4
            block  ;; label = @5
              local.get 3
              local.get 6
              i32.add
              local.tee 3
              i32.const 3
              i32.and
              br_if 0 (;@5;)
              local.get 4
              local.get 7
              i32.ge_u
              br_if 1 (;@4;)
              local.get 9
              local.get 1
              i32.add
              i32.const -4
              i32.add
              local.set 1
              loop  ;; label = @6
                local.get 7
                i32.const -4
                i32.add
                local.tee 7
                local.get 1
                i32.load
                i32.store
                local.get 1
                i32.const -4
                i32.add
                local.set 1
                local.get 4
                local.get 7
                i32.lt_u
                br_if 0 (;@6;)
                br 2 (;@4;)
              end
            end
            local.get 4
            local.get 7
            i32.ge_u
            br_if 0 (;@4;)
            local.get 3
            i32.const 3
            i32.shl
            local.tee 2
            i32.const 24
            i32.and
            local.set 5
            local.get 3
            i32.const -4
            i32.and
            local.tee 8
            i32.const -4
            i32.add
            local.set 1
            i32.const 0
            local.get 2
            i32.sub
            i32.const 24
            i32.and
            local.set 6
            local.get 8
            i32.load
            local.set 2
            loop  ;; label = @5
              local.get 7
              i32.const -4
              i32.add
              local.tee 7
              local.get 2
              local.get 6
              i32.shl
              local.get 1
              i32.load
              local.tee 2
              local.get 5
              i32.shr_u
              i32.or
              i32.store
              local.get 1
              i32.const -4
              i32.add
              local.set 1
              local.get 4
              local.get 7
              i32.lt_u
              br_if 0 (;@5;)
            end
          end
          local.get 9
          i32.const 3
          i32.and
          local.set 2
          local.get 3
          local.get 10
          i32.add
          local.set 3
          br 1 (;@2;)
        end
        block  ;; label = @3
          block  ;; label = @4
            local.get 2
            i32.const 16
            i32.ge_u
            br_if 0 (;@4;)
            local.get 0
            local.set 4
            br 1 (;@3;)
          end
          block  ;; label = @4
            local.get 0
            i32.const 0
            local.get 0
            i32.sub
            i32.const 3
            i32.and
            local.tee 10
            i32.add
            local.tee 9
            local.get 0
            i32.le_u
            br_if 0 (;@4;)
            local.get 10
            i32.const -1
            i32.add
            local.set 5
            local.get 0
            local.set 4
            local.get 1
            local.set 7
            block  ;; label = @5
              local.get 10
              i32.eqz
              br_if 0 (;@5;)
              local.get 10
              local.set 3
              local.get 0
              local.set 4
              local.get 1
              local.set 7
              loop  ;; label = @6
                local.get 4
                local.get 7
                i32.load8_u
                i32.store8
                local.get 7
                i32.const 1
                i32.add
                local.set 7
                local.get 4
                i32.const 1
                i32.add
                local.set 4
                local.get 3
                i32.const -1
                i32.add
                local.tee 3
                br_if 0 (;@6;)
              end
            end
            local.get 5
            i32.const 7
            i32.lt_u
            br_if 0 (;@4;)
            loop  ;; label = @5
              local.get 4
              local.get 7
              i32.load8_u
              i32.store8
              local.get 4
              i32.const 1
              i32.add
              local.get 7
              i32.const 1
              i32.add
              i32.load8_u
              i32.store8
              local.get 4
              i32.const 2
              i32.add
              local.get 7
              i32.const 2
              i32.add
              i32.load8_u
              i32.store8
              local.get 4
              i32.const 3
              i32.add
              local.get 7
              i32.const 3
              i32.add
              i32.load8_u
              i32.store8
              local.get 4
              i32.const 4
              i32.add
              local.get 7
              i32.const 4
              i32.add
              i32.load8_u
              i32.store8
              local.get 4
              i32.const 5
              i32.add
              local.get 7
              i32.const 5
              i32.add
              i32.load8_u
              i32.store8
              local.get 4
              i32.const 6
              i32.add
              local.get 7
              i32.const 6
              i32.add
              i32.load8_u
              i32.store8
              local.get 4
              i32.const 7
              i32.add
              local.get 7
              i32.const 7
              i32.add
              i32.load8_u
              i32.store8
              local.get 7
              i32.const 8
              i32.add
              local.set 7
              local.get 4
              i32.const 8
              i32.add
              local.tee 4
              local.get 9
              i32.ne
              br_if 0 (;@5;)
            end
          end
          local.get 9
          local.get 2
          local.get 10
          i32.sub
          local.tee 3
          i32.const -4
          i32.and
          local.tee 5
          i32.add
          local.set 4
          block  ;; label = @4
            block  ;; label = @5
              local.get 1
              local.get 10
              i32.add
              local.tee 7
              i32.const 3
              i32.and
              br_if 0 (;@5;)
              local.get 9
              local.get 4
              i32.ge_u
              br_if 1 (;@4;)
              local.get 7
              local.set 1
              loop  ;; label = @6
                local.get 9
                local.get 1
                i32.load
                i32.store
                local.get 1
                i32.const 4
                i32.add
                local.set 1
                local.get 9
                i32.const 4
                i32.add
                local.tee 9
                local.get 4
                i32.lt_u
                br_if 0 (;@6;)
                br 2 (;@4;)
              end
            end
            local.get 9
            local.get 4
            i32.ge_u
            br_if 0 (;@4;)
            local.get 7
            i32.const 3
            i32.shl
            local.tee 2
            i32.const 24
            i32.and
            local.set 10
            local.get 7
            i32.const -4
            i32.and
            local.tee 8
            i32.const 4
            i32.add
            local.set 1
            i32.const 0
            local.get 2
            i32.sub
            i32.const 24
            i32.and
            local.set 6
            local.get 8
            i32.load
            local.set 2
            loop  ;; label = @5
              local.get 9
              local.get 2
              local.get 10
              i32.shr_u
              local.get 1
              i32.load
              local.tee 2
              local.get 6
              i32.shl
              i32.or
              i32.store
              local.get 1
              i32.const 4
              i32.add
              local.set 1
              local.get 9
              i32.const 4
              i32.add
              local.tee 9
              local.get 4
              i32.lt_u
              br_if 0 (;@5;)
            end
          end
          local.get 3
          i32.const 3
          i32.and
          local.set 2
          local.get 7
          local.get 5
          i32.add
          local.set 1
        end
        local.get 4
        local.get 4
        local.get 2
        i32.add
        local.tee 9
        i32.ge_u
        br_if 1 (;@1;)
        local.get 2
        i32.const -1
        i32.add
        local.set 3
        block  ;; label = @3
          local.get 2
          i32.const 7
          i32.and
          local.tee 7
          i32.eqz
          br_if 0 (;@3;)
          loop  ;; label = @4
            local.get 4
            local.get 1
            i32.load8_u
            i32.store8
            local.get 1
            i32.const 1
            i32.add
            local.set 1
            local.get 4
            i32.const 1
            i32.add
            local.set 4
            local.get 7
            i32.const -1
            i32.add
            local.tee 7
            br_if 0 (;@4;)
          end
        end
        local.get 3
        i32.const 7
        i32.lt_u
        br_if 1 (;@1;)
        loop  ;; label = @3
          local.get 4
          local.get 1
          i32.load8_u
          i32.store8
          local.get 4
          i32.const 1
          i32.add
          local.get 1
          i32.const 1
          i32.add
          i32.load8_u
          i32.store8
          local.get 4
          i32.const 2
          i32.add
          local.get 1
          i32.const 2
          i32.add
          i32.load8_u
          i32.store8
          local.get 4
          i32.const 3
          i32.add
          local.get 1
          i32.const 3
          i32.add
          i32.load8_u
          i32.store8
          local.get 4
          i32.const 4
          i32.add
          local.get 1
          i32.const 4
          i32.add
          i32.load8_u
          i32.store8
          local.get 4
          i32.const 5
          i32.add
          local.get 1
          i32.const 5
          i32.add
          i32.load8_u
          i32.store8
          local.get 4
          i32.const 6
          i32.add
          local.get 1
          i32.const 6
          i32.add
          i32.load8_u
          i32.store8
          local.get 4
          i32.const 7
          i32.add
          local.get 1
          i32.const 7
          i32.add
          i32.load8_u
          i32.store8
          local.get 1
          i32.const 8
          i32.add
          local.set 1
          local.get 4
          i32.const 8
          i32.add
          local.tee 4
          local.get 9
          i32.ne
          br_if 0 (;@3;)
          br 2 (;@1;)
        end
      end
      local.get 4
      local.get 2
      i32.sub
      local.tee 7
      local.get 4
      i32.ge_u
      br_if 0 (;@1;)
      local.get 2
      i32.const -1
      i32.add
      local.set 9
      block  ;; label = @2
        local.get 2
        i32.const 3
        i32.and
        local.tee 1
        i32.eqz
        br_if 0 (;@2;)
        loop  ;; label = @3
          local.get 4
          i32.const -1
          i32.add
          local.tee 4
          local.get 3
          i32.const -1
          i32.add
          local.tee 3
          i32.load8_u
          i32.store8
          local.get 1
          i32.const -1
          i32.add
          local.tee 1
          br_if 0 (;@3;)
        end
      end
      local.get 9
      i32.const 3
      i32.lt_u
      br_if 0 (;@1;)
      local.get 3
      i32.const -4
      i32.add
      local.set 1
      loop  ;; label = @2
        local.get 4
        i32.const -1
        i32.add
        local.get 1
        i32.const 3
        i32.add
        i32.load8_u
        i32.store8
        local.get 4
        i32.const -2
        i32.add
        local.get 1
        i32.const 2
        i32.add
        i32.load8_u
        i32.store8
        local.get 4
        i32.const -3
        i32.add
        local.get 1
        i32.const 1
        i32.add
        i32.load8_u
        i32.store8
        local.get 4
        i32.const -4
        i32.add
        local.tee 4
        local.get 1
        i32.load8_u
        i32.store8
        local.get 1
        i32.const -4
        i32.add
        local.set 1
        local.get 7
        local.get 4
        i32.lt_u
        br_if 0 (;@2;)
      end
    end
    local.get 0)
  (func $memmove (type 6) (param i32 i32 i32) (result i32)
    local.get 0
    local.get 1
    local.get 2
    call $_ZN17compiler_builtins3mem7memmove17hd6c723cac9ad46b4E)
  (func $memcpy (type 6) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        i32.const 16
        i32.ge_u
        br_if 0 (;@2;)
        local.get 0
        local.set 3
        br 1 (;@1;)
      end
      block  ;; label = @2
        local.get 0
        i32.const 0
        local.get 0
        i32.sub
        i32.const 3
        i32.and
        local.tee 4
        i32.add
        local.tee 5
        local.get 0
        i32.le_u
        br_if 0 (;@2;)
        local.get 4
        i32.const -1
        i32.add
        local.set 6
        local.get 0
        local.set 3
        local.get 1
        local.set 7
        block  ;; label = @3
          local.get 4
          i32.eqz
          br_if 0 (;@3;)
          local.get 4
          local.set 8
          local.get 0
          local.set 3
          local.get 1
          local.set 7
          loop  ;; label = @4
            local.get 3
            local.get 7
            i32.load8_u
            i32.store8
            local.get 7
            i32.const 1
            i32.add
            local.set 7
            local.get 3
            i32.const 1
            i32.add
            local.set 3
            local.get 8
            i32.const -1
            i32.add
            local.tee 8
            br_if 0 (;@4;)
          end
        end
        local.get 6
        i32.const 7
        i32.lt_u
        br_if 0 (;@2;)
        loop  ;; label = @3
          local.get 3
          local.get 7
          i32.load8_u
          i32.store8
          local.get 3
          i32.const 1
          i32.add
          local.get 7
          i32.const 1
          i32.add
          i32.load8_u
          i32.store8
          local.get 3
          i32.const 2
          i32.add
          local.get 7
          i32.const 2
          i32.add
          i32.load8_u
          i32.store8
          local.get 3
          i32.const 3
          i32.add
          local.get 7
          i32.const 3
          i32.add
          i32.load8_u
          i32.store8
          local.get 3
          i32.const 4
          i32.add
          local.get 7
          i32.const 4
          i32.add
          i32.load8_u
          i32.store8
          local.get 3
          i32.const 5
          i32.add
          local.get 7
          i32.const 5
          i32.add
          i32.load8_u
          i32.store8
          local.get 3
          i32.const 6
          i32.add
          local.get 7
          i32.const 6
          i32.add
          i32.load8_u
          i32.store8
          local.get 3
          i32.const 7
          i32.add
          local.get 7
          i32.const 7
          i32.add
          i32.load8_u
          i32.store8
          local.get 7
          i32.const 8
          i32.add
          local.set 7
          local.get 3
          i32.const 8
          i32.add
          local.tee 3
          local.get 5
          i32.ne
          br_if 0 (;@3;)
        end
      end
      local.get 5
      local.get 2
      local.get 4
      i32.sub
      local.tee 8
      i32.const -4
      i32.and
      local.tee 6
      i32.add
      local.set 3
      block  ;; label = @2
        block  ;; label = @3
          local.get 1
          local.get 4
          i32.add
          local.tee 7
          i32.const 3
          i32.and
          br_if 0 (;@3;)
          local.get 5
          local.get 3
          i32.ge_u
          br_if 1 (;@2;)
          local.get 7
          local.set 1
          loop  ;; label = @4
            local.get 5
            local.get 1
            i32.load
            i32.store
            local.get 1
            i32.const 4
            i32.add
            local.set 1
            local.get 5
            i32.const 4
            i32.add
            local.tee 5
            local.get 3
            i32.lt_u
            br_if 0 (;@4;)
            br 2 (;@2;)
          end
        end
        local.get 5
        local.get 3
        i32.ge_u
        br_if 0 (;@2;)
        local.get 7
        i32.const 3
        i32.shl
        local.tee 2
        i32.const 24
        i32.and
        local.set 4
        local.get 7
        i32.const -4
        i32.and
        local.tee 9
        i32.const 4
        i32.add
        local.set 1
        i32.const 0
        local.get 2
        i32.sub
        i32.const 24
        i32.and
        local.set 10
        local.get 9
        i32.load
        local.set 2
        loop  ;; label = @3
          local.get 5
          local.get 2
          local.get 4
          i32.shr_u
          local.get 1
          i32.load
          local.tee 2
          local.get 10
          i32.shl
          i32.or
          i32.store
          local.get 1
          i32.const 4
          i32.add
          local.set 1
          local.get 5
          i32.const 4
          i32.add
          local.tee 5
          local.get 3
          i32.lt_u
          br_if 0 (;@3;)
        end
      end
      local.get 8
      i32.const 3
      i32.and
      local.set 2
      local.get 7
      local.get 6
      i32.add
      local.set 1
    end
    block  ;; label = @1
      local.get 3
      local.get 3
      local.get 2
      i32.add
      local.tee 5
      i32.ge_u
      br_if 0 (;@1;)
      local.get 2
      i32.const -1
      i32.add
      local.set 8
      block  ;; label = @2
        local.get 2
        i32.const 7
        i32.and
        local.tee 7
        i32.eqz
        br_if 0 (;@2;)
        loop  ;; label = @3
          local.get 3
          local.get 1
          i32.load8_u
          i32.store8
          local.get 1
          i32.const 1
          i32.add
          local.set 1
          local.get 3
          i32.const 1
          i32.add
          local.set 3
          local.get 7
          i32.const -1
          i32.add
          local.tee 7
          br_if 0 (;@3;)
        end
      end
      local.get 8
      i32.const 7
      i32.lt_u
      br_if 0 (;@1;)
      loop  ;; label = @2
        local.get 3
        local.get 1
        i32.load8_u
        i32.store8
        local.get 3
        i32.const 1
        i32.add
        local.get 1
        i32.const 1
        i32.add
        i32.load8_u
        i32.store8
        local.get 3
        i32.const 2
        i32.add
        local.get 1
        i32.const 2
        i32.add
        i32.load8_u
        i32.store8
        local.get 3
        i32.const 3
        i32.add
        local.get 1
        i32.const 3
        i32.add
        i32.load8_u
        i32.store8
        local.get 3
        i32.const 4
        i32.add
        local.get 1
        i32.const 4
        i32.add
        i32.load8_u
        i32.store8
        local.get 3
        i32.const 5
        i32.add
        local.get 1
        i32.const 5
        i32.add
        i32.load8_u
        i32.store8
        local.get 3
        i32.const 6
        i32.add
        local.get 1
        i32.const 6
        i32.add
        i32.load8_u
        i32.store8
        local.get 3
        i32.const 7
        i32.add
        local.get 1
        i32.const 7
        i32.add
        i32.load8_u
        i32.store8
        local.get 1
        i32.const 8
        i32.add
        local.set 1
        local.get 3
        i32.const 8
        i32.add
        local.tee 3
        local.get 5
        i32.ne
        br_if 0 (;@2;)
      end
    end
    local.get 0)
  (func $memset (type 6) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        i32.const 16
        i32.ge_u
        br_if 0 (;@2;)
        local.get 0
        local.set 3
        br 1 (;@1;)
      end
      block  ;; label = @2
        local.get 0
        i32.const 0
        local.get 0
        i32.sub
        i32.const 3
        i32.and
        local.tee 4
        i32.add
        local.tee 5
        local.get 0
        i32.le_u
        br_if 0 (;@2;)
        local.get 4
        i32.const -1
        i32.add
        local.set 6
        local.get 0
        local.set 3
        block  ;; label = @3
          local.get 4
          i32.eqz
          br_if 0 (;@3;)
          local.get 4
          local.set 7
          local.get 0
          local.set 3
          loop  ;; label = @4
            local.get 3
            local.get 1
            i32.store8
            local.get 3
            i32.const 1
            i32.add
            local.set 3
            local.get 7
            i32.const -1
            i32.add
            local.tee 7
            br_if 0 (;@4;)
          end
        end
        local.get 6
        i32.const 7
        i32.lt_u
        br_if 0 (;@2;)
        loop  ;; label = @3
          local.get 3
          local.get 1
          i32.store8
          local.get 3
          i32.const 7
          i32.add
          local.get 1
          i32.store8
          local.get 3
          i32.const 6
          i32.add
          local.get 1
          i32.store8
          local.get 3
          i32.const 5
          i32.add
          local.get 1
          i32.store8
          local.get 3
          i32.const 4
          i32.add
          local.get 1
          i32.store8
          local.get 3
          i32.const 3
          i32.add
          local.get 1
          i32.store8
          local.get 3
          i32.const 2
          i32.add
          local.get 1
          i32.store8
          local.get 3
          i32.const 1
          i32.add
          local.get 1
          i32.store8
          local.get 3
          i32.const 8
          i32.add
          local.tee 3
          local.get 5
          i32.ne
          br_if 0 (;@3;)
        end
      end
      block  ;; label = @2
        local.get 5
        local.get 5
        local.get 2
        local.get 4
        i32.sub
        local.tee 2
        i32.const -4
        i32.and
        i32.add
        local.tee 3
        i32.ge_u
        br_if 0 (;@2;)
        local.get 1
        i32.const 255
        i32.and
        i32.const 16843009
        i32.mul
        local.set 7
        loop  ;; label = @3
          local.get 5
          local.get 7
          i32.store
          local.get 5
          i32.const 4
          i32.add
          local.tee 5
          local.get 3
          i32.lt_u
          br_if 0 (;@3;)
        end
      end
      local.get 2
      i32.const 3
      i32.and
      local.set 2
    end
    block  ;; label = @1
      local.get 3
      local.get 3
      local.get 2
      i32.add
      local.tee 7
      i32.ge_u
      br_if 0 (;@1;)
      local.get 2
      i32.const -1
      i32.add
      local.set 4
      block  ;; label = @2
        local.get 2
        i32.const 7
        i32.and
        local.tee 5
        i32.eqz
        br_if 0 (;@2;)
        loop  ;; label = @3
          local.get 3
          local.get 1
          i32.store8
          local.get 3
          i32.const 1
          i32.add
          local.set 3
          local.get 5
          i32.const -1
          i32.add
          local.tee 5
          br_if 0 (;@3;)
        end
      end
      local.get 4
      i32.const 7
      i32.lt_u
      br_if 0 (;@1;)
      loop  ;; label = @2
        local.get 3
        local.get 1
        i32.store8
        local.get 3
        i32.const 7
        i32.add
        local.get 1
        i32.store8
        local.get 3
        i32.const 6
        i32.add
        local.get 1
        i32.store8
        local.get 3
        i32.const 5
        i32.add
        local.get 1
        i32.store8
        local.get 3
        i32.const 4
        i32.add
        local.get 1
        i32.store8
        local.get 3
        i32.const 3
        i32.add
        local.get 1
        i32.store8
        local.get 3
        i32.const 2
        i32.add
        local.get 1
        i32.store8
        local.get 3
        i32.const 1
        i32.add
        local.get 1
        i32.store8
        local.get 3
        i32.const 8
        i32.add
        local.tee 3
        local.get 7
        i32.ne
        br_if 0 (;@2;)
      end
    end
    local.get 0)
  (table (;0;) 64 64 funcref)
  (memory (;0;) 17)
  (global $__stack_pointer (mut i32) (i32.const 1048576))
  (global (;1;) i32 (i32.const 1059596))
  (global (;2;) i32 (i32.const 1059600))
  (export "memory" (memory 0))
  (export "mandelbrot" (func $mandelbrot))
  (export "main" (func $main))
  (export "__data_end" (global 1))
  (export "__heap_base" (global 2))
  (elem (;0;) (i32.const 1) func $_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17hb0525cf1cd8205dbE.llvm.12408529821376129927 $_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h324098578815f6c0E.llvm.12408529821376129927 $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h02cb3d0998e89334E $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17ha4277154763c2c00E $_ZN83_$LT$rayon_core..job..StackJob$LT$L$C$F$C$R$GT$$u20$as$u20$rayon_core..job..Job$GT$7execute17h2226c5c9c3f32125E $_ZN10mandelbrot4main17haebc31b42b6c82b6E $_ZN83_$LT$rayon_core..job..StackJob$LT$L$C$F$C$R$GT$$u20$as$u20$rayon_core..job..Job$GT$7execute17hf4bf52d52279e47bE $_ZN83_$LT$rayon_core..job..StackJob$LT$L$C$F$C$R$GT$$u20$as$u20$rayon_core..job..Job$GT$7execute17h8a40026598c18a69E $_ZN15crossbeam_epoch8deferred8Deferred3new4call17hba3722852837b63cE.llvm.2793606751137166678 $_ZN4core3ptr53drop_in_place$LT$rayon_core..ThreadPoolBuildError$GT$17h765bdb48184b1c4cE.llvm.16897747005057573272 $_ZN69_$LT$rayon_core..ThreadPoolBuildError$u20$as$u20$core..fmt..Debug$GT$3fmt17h3c8116c51c896825E.llvm.16897747005057573272 $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h7a03664ec6aefbbbE $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h053a31c8c3f17766E $_ZN15crossbeam_epoch8deferred8Deferred3new4call17h2992095b69a7b6f4E.llvm.6263538152696972293 $_ZN4core3ptr230drop_in_place$LT$std..thread..Builder..spawn_unchecked_$LT$$LT$rayon_core..registry..DefaultSpawn$u20$as$u20$rayon_core..registry..ThreadSpawn$GT$..spawn..$u7b$$u7b$closure$u7d$$u7d$$C$$LP$$RP$$GT$..$u7b$$u7b$closure$u7d$$u7d$$GT$17hc755cde14436e666E $_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h44a1b090b5e82e47E $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h30bcf172ba43f522E $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h03850c4c46fb1eb2E $_ZN15crossbeam_epoch8deferred8Deferred3new4call17h11a4a864381fbb19E.llvm.14801380999711903380 $_ZN15crossbeam_epoch8deferred8Deferred3new4call17h10d9322f5bc84455E.llvm.14801380999711903380 $_ZN15crossbeam_epoch8deferred8Deferred5NO_OP10no_op_call17h95b2cac4a016c451E.llvm.6279718309794500740 $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17he1d3bba66865ae66E $_ZN3std5alloc24default_alloc_error_hook17hb6719f23c72b7373E $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h76b2378a35984c0bE $_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..Debug$u20$for$u20$usize$GT$3fmt17h8418712dcb2dbdd0E $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h16d7108310351cefE $_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17hea3cf74d7f6052eaE $_ZN58_$LT$alloc..string..String$u20$as$u20$core..fmt..Write$GT$9write_str17habb26b011a335421E $_ZN58_$LT$alloc..string..String$u20$as$u20$core..fmt..Write$GT$10write_char17h01d044b6dc206b9eE $_ZN4core3fmt5Write9write_fmt17h8c80195cd11d832dE $_ZN4core3ptr48drop_in_place$LT$alloc..ffi..c_str..NulError$GT$17h4cf020dd6404566eE $_ZN64_$LT$alloc..ffi..c_str..NulError$u20$as$u20$core..fmt..Debug$GT$3fmt17ha182f484657bd2b4E $_ZN4core3fmt3num50_$LT$impl$u20$core..fmt..Debug$u20$for$u20$i32$GT$3fmt17h5ae1eb5c912fa259E $_ZN62_$LT$std..io..error..ErrorKind$u20$as$u20$core..fmt..Debug$GT$3fmt17hc9d9a0174449b0bfE $_ZN58_$LT$alloc..string..String$u20$as$u20$core..fmt..Debug$GT$3fmt17h3e6b0c9880e0a14aE $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hd222f1b27591d1e4E $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hcf27825cef0723b2E $_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17h0ef14a6024109e55E $_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17h4c46e3b945c67ec3E $_ZN92_$LT$std..panicking..begin_panic_handler..StaticStrPayload$u20$as$u20$core..fmt..Display$GT$3fmt17h7cb955cbcfed38e6E $_ZN99_$LT$std..panicking..begin_panic_handler..StaticStrPayload$u20$as$u20$core..panic..PanicPayload$GT$8take_box17h3a323dbea18dea96E $_ZN99_$LT$std..panicking..begin_panic_handler..StaticStrPayload$u20$as$u20$core..panic..PanicPayload$GT$3get17h85e47ee294bc51aeE $_ZN99_$LT$std..panicking..begin_panic_handler..StaticStrPayload$u20$as$u20$core..panic..PanicPayload$GT$6as_str17h12728f070b9d6551E $_ZN4core3ptr77drop_in_place$LT$std..panicking..begin_panic_handler..FormatStringPayload$GT$17h3ce79a741b975d36E $_ZN95_$LT$std..panicking..begin_panic_handler..FormatStringPayload$u20$as$u20$core..fmt..Display$GT$3fmt17hd74758c7e313799bE $_ZN102_$LT$std..panicking..begin_panic_handler..FormatStringPayload$u20$as$u20$core..panic..PanicPayload$GT$8take_box17hd73901fc3ea6a706E $_ZN102_$LT$std..panicking..begin_panic_handler..FormatStringPayload$u20$as$u20$core..panic..PanicPayload$GT$3get17h624ff36aab1cfa5dE $_ZN4core5panic12PanicPayload6as_str17h7ffa20843f2d9518E $_ZN4core3ptr71drop_in_place$LT$std..panicking..rust_panic_without_hook..RewrapBox$GT$17hf26041a55470cdfdE $_ZN89_$LT$std..panicking..rust_panic_without_hook..RewrapBox$u20$as$u20$core..fmt..Display$GT$3fmt17h8f45bfa2abb35ee5E $_ZN96_$LT$std..panicking..rust_panic_without_hook..RewrapBox$u20$as$u20$core..panic..PanicPayload$GT$8take_box17h659baeadd9cb1069E $_ZN96_$LT$std..panicking..rust_panic_without_hook..RewrapBox$u20$as$u20$core..panic..PanicPayload$GT$3get17hc81c0be5a0b540ffE $_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17h419978c7402a9818E $_ZN69_$LT$core..alloc..layout..LayoutError$u20$as$u20$core..fmt..Debug$GT$3fmt17hdc389e2f810cfa03E $_ZN63_$LT$core..cell..BorrowMutError$u20$as$u20$core..fmt..Debug$GT$3fmt17h85594013aacd41e5E $_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hb26ba63d6930e1beE $_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h8ef67506d1a750e3E $_ZN59_$LT$core..fmt..Arguments$u20$as$u20$core..fmt..Display$GT$3fmt17h710b3907e5e9fa00E $_ZN71_$LT$core..ops..range..Range$LT$Idx$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h4e93df056c33b284E $_ZN41_$LT$char$u20$as$u20$core..fmt..Debug$GT$3fmt17h6ec13b7460ecb1beE $_ZN68_$LT$core..fmt..builders..PadAdapter$u20$as$u20$core..fmt..Write$GT$9write_str17h50c1b995f7ff56e5E $_ZN68_$LT$core..fmt..builders..PadAdapter$u20$as$u20$core..fmt..Write$GT$10write_char17habd2bb5515050163E $_ZN4core3fmt5Write9write_fmt17hca9ef98f6e4b87ecE)
  (data $.rodata (i32.const 1048576) "/home/chikuwait/.cargo/registry/src/index.crates.io-6f17d22bba15001f/rayon-1.10.0/src/slice/chunks.rschunk size must be non-zeroe\00\10\00\1b\00\00\00\00\00\10\00e\00\00\00\fd\00\00\00\14\00\00\00\00\00\00\00\04\00\00\00\04\00\00\00\01\00\00\00\02\00\00\00\02\00\00\00/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/core/src/iter/traits/iterator.rs\b0\00\10\00X\00\00\00\b3\07\00\00\09\00\00\00\00\00\00\00\04\00\00\00\04\00\00\00\03\00\00\00\00\00\00\00\04\00\00\00\04\00\00\00\04\00\00\00internal error: entered unreachable code/home/chikuwait/.cargo/registry/src/index.crates.io-6f17d22bba15001f/rayon-core-1.12.1/src/job.rs\00\00\00`\01\10\00a\00\00\00\e6\00\00\00 \00\00\00`\01\10\00a\00\00\00f\00\00\00 \00\00\00assertion failed: injected && !worker_thread.is_null()/home/chikuwait/.cargo/registry/src/index.crates.io-6f17d22bba15001f/rayon-core-1.12.1/src/registry.rs\1a\02\10\00f\00\00\00\09\02\00\00\15\00\00\00\1a\02\10\00f\00\00\00\22\02\00\00\11\00\00\00`\01\10\00a\00\00\00w\00\00\00.\00\00\00/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/core/src/iter/traits/iterator.rs\b0\02\10\00X\00\00\00\b3\07\00\00\09\00\00\00mandelbrot.rs\00\00\00\18\03\10\00\0d\00\00\00\a5\00\00\00\14\00\00\00\18\03\10\00\0d\00\00\00\a7\00\00\00\09\00\00\00\18\03\10\00\0d\00\00\00\a8\00\00\00\09\00\00\00\18\03\10\00\0d\00\00\00\ae\00\00\00#\00\00\00cannot recursively acquire mutexh\03\10\00 \00\00\00\00/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/std/src/sys/sync/mutex/no_threads.rs\00\00\00\91\03\10\00\5c\00\00\00\14\00\00\00\09\00\00\00/home/chikuwait/.cargo/registry/src/index.crates.io-6f17d22bba15001f/crossbeam-epoch-0.9.18/src/internal.rs\00\00\04\10\00k\00\00\00\81\01\00\009\00\00\00/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/alloc/src/slice.rs\00\00|\04\10\00J\00\00\00\9f\00\00\00\19\00\00\00/home/chikuwait/.cargo/registry/src/index.crates.io-6f17d22bba15001f/rayon-core-1.12.1/src/registry.rs\00\00\d8\04\10\00f\00\00\00\c0\00\00\00\16\00\00\00\0a\00\00\00\08\00\00\00\04\00\00\00\0b\00\00\00The global thread pool has not been initialized.\d8\04\10\00f\00\00\00\a8\00\00\00\0a\00\00\00\d8\04\10\00f\00\00\00+\01\00\006\00\00\00\d8\04\10\00f\00\00\002\03\00\00/\00\00\00\d8\04\10\00f\00\00\008\03\00\00*\00\00\00\d8\04\10\00f\00\00\00\8f\03\00\00&\00\00\00\00\00\00\00\04\00\00\00\04\00\00\00\0c\00\00\00ThreadPoolBuildErrorkindassertion failed: t.get().eq(&(self as *const _))/home/chikuwait/.cargo/registry/src/index.crates.io-6f17d22bba15001f/rayon-core-1.12.1/src/registry.rs\009\06\10\00f\00\00\00\ad\02\00\00\0d\00\00\00assertion failed: t.get().is_null()\009\06\10\00f\00\00\00\c0\02\00\00\0d\00\00\00Once instance has previously been poisoned\00\00\e4\06\10\00*\00\00\00one-time initialization may not be performed recursively\18\07\10\008\00\00\00/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/std/src/sync/once.rsX\07\10\00L\00\00\00\9e\00\00\002\00\00\00cannot recursively acquire mutex\b4\07\10\00 \00\00\00\00/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/std/src/sys/sync/mutex/no_threads.rs\00\00\00\dd\07\10\00\5c\00\00\00\14\00\00\00\09\00\00\00\00\00\00\00\04\00\00\00\04\00\00\00\0d\00\00\00/home/chikuwait/.cargo/registry/src/index.crates.io-6f17d22bba15001f/rayon-core-1.12.1/src/registry.rs\00\00\5c\08\10\00f\00\00\00u\03\00\00#\00\00\00/home/chikuwait/.cargo/registry/src/index.crates.io-6f17d22bba15001f/rayon-core-1.12.1/src/sleep/mod.rs/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/core/src/iter/traits/iterator.rs\00;\09\10\00X\00\00\00\b3\07\00\00\09\00\00\00\d4\08\10\00g\00\00\00\83\00\00\004\00\00\00\d4\08\10\00g\00\00\00\22\01\00\004\00\00\00/home/chikuwait/.cargo/registry/src/index.crates.io-6f17d22bba15001f/crossbeam-deque-0.8.6/src/deque.rs\00\c4\09\10\00g\00\00\00\7f\05\00\00C\00\00\00/home/chikuwait/.cargo/registry/src/index.crates.io-6f17d22bba15001f/crossbeam-epoch-0.9.18/src/internal.rs\00<\0a\10\00k\00\00\00\81\01\00\009\00\00\00/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/alloc/src/vec/spec_from_iter_nested.rs\00\00\b8\0a\10\00^\00\00\004\00\00\00\05\00\00\00\0f\00\00\00P\00\00\00\04\00\00\00\10\00\00\00/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/core/src/iter/traits/iterator.rs8\0b\10\00X\00\00\00\b3\07\00\00\09\00\00\00GlobalPoolAlreadyInitializedCurrentThreadAlreadyInPool\00\00\00\00\00\00\04\00\00\00\04\00\00\00\11\00\00\00IOError/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/alloc/src/raw_vec.rs\00\ef\0b\10\00L\00\00\00+\02\00\00\11\00\00\00Once instance has previously been poisoned\00\00L\0c\10\00*\00\00\00one-time initialization may not be performed recursively\80\0c\10\008\00\00\00/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/std/src/sync/once.rs\c0\0c\10\00L\00\00\00\9e\00\00\002\00\00\00/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/core/src/sync/atomic.rsthere is no such thing as a release failure ordering\00k\0d\10\004\00\00\00\1c\0d\10\00O\00\00\00f\0d\00\00\1d\00\00\00there is no such thing as an acquire-release failure ordering\00\00\00\b8\0d\10\00=\00\00\00\1c\0d\10\00O\00\00\00e\0d\00\00\1c\00\00\00\00\00\00\00\04\00\00\00\04\00\00\00\12\00\00\00unaligned pointer\00\00\00 \0e\10\00\11\00\00\00\00\00\00\00/home/chikuwait/.cargo/registry/src/index.crates.io-6f17d22bba15001f/crossbeam-epoch-0.9.18/src/atomic.rs\00\00\00@\0e\10\00i\00\00\00q\00\00\00\05\00\00\00\01\00\00\00/home/chikuwait/.cargo/registry/src/index.crates.io-6f17d22bba15001f/crossbeam-epoch-0.9.18/src/sync/list.rs\c0\0e\10\00l\00\00\00\e2\00\00\00\11\00\00\00/home/chikuwait/.cargo/registry/src/index.crates.io-6f17d22bba15001f/crossbeam-epoch-0.9.18/src/sync/once_lock.rs\00\00\00<\0f\10\00q\00\00\00B\00\00\00\13\00\00\00/home/chikuwait/.cargo/registry/src/index.crates.io-6f17d22bba15001f/crossbeam-epoch-0.9.18/src/internal.rs\00\c0\0f\10\00k\00\00\00w\00\00\00,\00\00\00\15\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\c0\0f\10\00k\00\00\00\81\01\00\009\00\00\00\00\00\00\00\00\00\00\00\04\00\00\00\04\00\00\00\18\00\00\00/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/alloc/src/slice.rs\00\00p\10\10\00J\00\00\00\9f\00\00\00\19\00\00\00/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/alloc/src/string.rs\00\cc\10\10\00K\00\00\00\8d\05\00\00\1b\00\00\00/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/alloc/src/raw_vec.rs(\11\10\00L\00\00\00+\02\00\00\11\00\00\00\00\00\00\00\04\00\00\00\04\00\00\00\19\00\00\00\00\00\00\00\04\00\00\00\04\00\00\00\1a\00\00\00NulError\1b\00\00\00\0c\00\00\00\04\00\00\00\1c\00\00\00\1d\00\00\00\1e\00\00\00/rust/deps/dlmalloc-0.2.7/src/dlmalloc.rsassertion failed: psize >= size + min_overhead\00\c4\11\10\00)\00\00\00\a8\04\00\00\09\00\00\00assertion failed: psize <= size + max_overhead\00\00\c4\11\10\00)\00\00\00\ae\04\00\00\0d\00\00\00std/src/rt.rs\00\00\00l\12\10\00\0d\00\00\00\90\00\00\00\0d\00\00\00too many running threads in thread scope\8c\12\10\00(\00\00\00std/src/thread/scoped.rs\bc\12\10\00\18\00\00\008\00\00\00\09\00\00\00/rustc/e71f9a9a98b0faf423844bf0ba7438f29dc27d58/library/core/src/iter/traits/iterator.rs\e4\12\10\00X\00\00\00\b3\07\00\00\09\00\00\00std/src/thread/mod.rsfailed to generate unique thread ID: bitspace exhausteda\13\10\007\00\00\00L\13\10\00\15\00\00\00\aa\04\00\00\0d\00\00\00\1f\00\00\00\10\00\00\00\04\00\00\00 \00\00\00thread name may not contain interior null bytes\00L\13\10\00\15\00\00\00\06\05\00\00(\00\00\00main\00operation not supported on this platform\00\00\00\05\14\10\00(\00\00\00$\00\00\00\00\00\00\00\02\00\00\000\14\10\00Os\00\00\00\00\00\00\04\00\00\00\04\00\00\00!\00\00\00code\00\00\00\00\01\00\00\00\01\00\00\00\22\00\00\00kind\1b\00\00\00\0c\00\00\00\04\00\00\00#\00\00\00messageKindError\00\00\00\00\08\00\00\00\04\00\00\00$\00\00\00\00\00\00\00\04\00\00\00\04\00\00\00%\00\00\00Customerrorstd/src/io/stdio.rs\00\00\bf\14\10\00\13\00\00\00\b9\02\00\00\13\00\00\00std/src/sync/once.rs\e4\14\10\00\14\00\00\00\9e\00\00\002\00\00\00\e4\14\10\00\14\00\00\00\d9\00\00\00\14\00\00\00memory allocation of  bytes failed\00\00\18\15\10\00\15\00\00\00-\15\10\00\0d\00\00\00std/src/alloc.rsL\15\10\00\10\00\00\00c\01\00\00\09\00\00\00\1b\00\00\00\0c\00\00\00\04\00\00\00&\00\00\00\00\00\00\00\08\00\00\00\04\00\00\00'\00\00\00\00\00\00\00\08\00\00\00\04\00\00\00(\00\00\00)\00\00\00*\00\00\00+\00\00\00,\00\00\00\10\00\00\00\04\00\00\00-\00\00\00.\00\00\00/\00\00\000\00\00\00Box<dyn Any>1\00\00\00\08\00\00\00\04\00\00\002\00\00\003\00\00\004\00\00\000\00\00\00\00\00\00\00\00\00\00\00\01\00\00\005\00\00\00NotFoundPermissionDeniedConnectionRefusedConnectionResetHostUnreachableNetworkUnreachableConnectionAbortedNotConnectedAddrInUseAddrNotAvailableNetworkDownBrokenPipeAlreadyExistsWouldBlockNotADirectoryIsADirectoryDirectoryNotEmptyReadOnlyFilesystemFilesystemLoopStaleNetworkFileHandleInvalidInputInvalidDataTimedOutWriteZeroStorageFullNotSeekableFilesystemQuotaExceededFileTooLargeResourceBusyExecutableFileBusyDeadlockCrossesDevicesTooManyLinksInvalidFilenameArgumentListTooLongInterruptedUnsupportedUnexpectedEofOutOfMemoryInProgressOtherUncategorizedoperation successfulcondvar wait not supported\00\008\18\10\00\1a\00\00\00std/src/sys/sync/condvar/no_threads.rs\00\00\5c\18\10\00&\00\00\00\14\00\00\00\09\00\00\00Once instance has previously been poisoned\00\00\94\18\10\00*\00\00\00one-time initialization may not be performed recursively\c8\18\10\008\00\00\00\08\00\00\00\10\00\00\00\11\00\00\00\0f\00\00\00\0f\00\00\00\12\00\00\00\11\00\00\00\0c\00\00\00\09\00\00\00\10\00\00\00\0b\00\00\00\0a\00\00\00\0d\00\00\00\0a\00\00\00\0d\00\00\00\0c\00\00\00\11\00\00\00\12\00\00\00\0e\00\00\00\16\00\00\00\0c\00\00\00\0b\00\00\00\08\00\00\00\09\00\00\00\0b\00\00\00\0b\00\00\00\17\00\00\00\0c\00\00\00\0c\00\00\00\12\00\00\00\08\00\00\00\0e\00\00\00\0c\00\00\00\0f\00\00\00\13\00\00\00\0b\00\00\00\0b\00\00\00\0d\00\00\00\0b\00\00\00\0a\00\00\00\05\00\00\00\0d\00\00\00\fc\15\10\00\04\16\10\00\14\16\10\00%\16\10\004\16\10\00C\16\10\00U\16\10\00f\16\10\00r\16\10\00{\16\10\00\8b\16\10\00\96\16\10\00\a0\16\10\00\ad\16\10\00\b7\16\10\00\c4\16\10\00\d0\16\10\00\e1\16\10\00\f3\16\10\00\01\17\10\00\17\17\10\00#\17\10\00.\17\10\006\17\10\00?\17\10\00J\17\10\00U\17\10\00l\17\10\00x\17\10\00\84\17\10\00\96\17\10\00\9e\17\10\00\ac\17\10\00\b8\17\10\00\c7\17\10\00\da\17\10\00\e5\17\10\00\f0\17\10\00\fd\17\10\00\08\18\10\00\12\18\10\00\17\18\10\00LayoutErrorcapacity overflowc\1a\10\00\11\00\00\00alloc/src/ffi/c_str.rs\00\00|\1a\10\00\16\00\00\00Y\01\00\00\0b\00\00\00\00\00\00\00\00\00\00\00\01\00\00\006\00\00\00called `Result::unwrap()` on an `Err` valuealloc/src/sync.rs\df\1a\10\00\11\00\00\00q\01\00\002\00\00\00)..0123456789abcdefBorrowMutErroralready borrowed: \00!\1b\10\00\12\00\00\00[called `Option::unwrap()` on a `None` valueindex out of bounds: the len is  but the index is \00\00h\1b\10\00 \00\00\00\88\1b\10\00\12\00\00\00==!=matchesassertion `left  right` failed\0a  left: \0a right: \00\b7\1b\10\00\10\00\00\00\c7\1b\10\00\17\00\00\00\de\1b\10\00\09\00\00\00 right` failed: \0a  left: \00\00\00\b7\1b\10\00\10\00\00\00\00\1c\10\00\10\00\00\00\10\1c\10\00\09\00\00\00\de\1b\10\00\09\00\00\00: \00\00\01\00\00\00\00\00\00\00<\1c\10\00\02\00\00\00\00\00\00\00\0c\00\00\00\04\00\00\00=\00\00\00>\00\00\00?\00\00\00     { ,  {\0a,\0a} }((\0a,\0a]0x00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899core/src/fmt/mod.rsfalsetrue\00\00\00I\1d\10\00\13\00\00\00\a6\09\00\00&\00\00\00I\1d\10\00\13\00\00\00\af\09\00\00\1a\00\00\00core/src/str/mod.rs[...]begin <= end ( <= ) when slicing ``\00\a0\1d\10\00\0e\00\00\00\ae\1d\10\00\04\00\00\00\b2\1d\10\00\10\00\00\00\c2\1d\10\00\01\00\00\00byte index  is not a char boundary; it is inside  (bytes ) of `\00\e4\1d\10\00\0b\00\00\00\ef\1d\10\00&\00\00\00\15\1e\10\00\08\00\00\00\1d\1e\10\00\06\00\00\00\c2\1d\10\00\01\00\00\00 is out of bounds of `\00\00\e4\1d\10\00\0b\00\00\00L\1e\10\00\16\00\00\00\c2\1d\10\00\01\00\00\00\88\1d\10\00\13\00\00\00\f4\00\00\00,\00\00\00core/src/unicode/printable.rs\00\00\00\8c\1e\10\00\1d\00\00\00\1a\00\00\006\00\00\00\8c\1e\10\00\1d\00\00\00\0a\00\00\00+\00\00\00\00\06\01\01\03\01\04\02\05\07\07\02\08\08\09\02\0a\05\0b\02\0e\04\10\01\11\02\12\05\13\1c\14\01\15\02\17\02\19\0d\1c\05\1d\08\1f\01$\01j\04k\02\af\03\b1\02\bc\02\cf\02\d1\02\d4\0c\d5\09\d6\02\d7\02\da\01\e0\05\e1\02\e7\04\e8\02\ee \f0\04\f8\02\fa\04\fb\01\0c';>NO\8f\9e\9e\9f{\8b\93\96\a2\b2\ba\86\b1\06\07\096=>V\f3\d0\d1\04\14\1867VW\7f\aa\ae\af\bd5\e0\12\87\89\8e\9e\04\0d\0e\11\12)14:EFIJNOde\8a\8c\8d\8f\b6\c1\c3\c4\c6\cb\d6\5c\b6\b7\1b\1c\07\08\0a\0b\14\1769:\a8\a9\d8\d9\097\90\91\a8\07\0a;>fi\8f\92\11o_\bf\ee\efZb\f4\fc\ffST\9a\9b./'(U\9d\a0\a1\a3\a4\a7\a8\ad\ba\bc\c4\06\0b\0c\15\1d:?EQ\a6\a7\cc\cd\a0\07\19\1a\22%>?\e7\ec\ef\ff\c5\c6\04 #%&(38:HJLPSUVXZ\5c^`cefksx}\7f\8a\a4\aa\af\b0\c0\d0\ae\afno\dd\de\93^\22{\05\03\04-\03f\03\01/.\80\82\1d\031\0f\1c\04$\09\1e\05+\05D\04\0e*\80\aa\06$\04$\04(\084\0bN\034\0c\817\09\16\0a\08\18;E9\03c\08\090\16\05!\03\1b\05\01@8\04K\05/\04\0a\07\09\07@ '\04\0c\096\03:\05\1a\07\04\0c\07PI73\0d3\07.\08\0a\06&\03\1d\08\02\80\d0R\10\037,\08*\16\1a&\1c\14\17\09N\04$\09D\0d\19\07\0a\06H\08'\09u\0bB>*\06;\05\0a\06Q\06\01\05\10\03\05\0bY\08\02\1db\1eH\08\0a\80\a6^\22E\0b\0a\06\0d\13:\06\0a\06\14\1c,\04\17\80\b9<dS\0cH\09\0aFE\1bH\08S\0dI\07\0a\80\b6\22\0e\0a\06F\0a\1d\03GI7\03\0e\08\0a\069\07\0a\816\19\07;\03\1dU\01\0f2\0d\83\9bfu\0b\80\c4\8aLc\0d\840\10\16\0a\8f\9b\05\82G\9a\b9:\86\c6\829\07*\04\5c\06&\0aF\0a(\05\13\81\b0:\80\c6[eK\049\07\11@\05\0b\02\0e\97\f8\08\84\d6)\0a\a2\e7\813\0f\01\1d\06\0e\04\08\81\8c\89\04k\05\0d\03\09\07\10\8f`\80\fa\06\81\b4LG\09t<\80\f6\0as\08p\15Fz\14\0c\14\0cW\09\19\80\87\81G\03\85B\0f\15\84P\1f\06\06\80\d5+\05>!\01p-\03\1a\04\02\81@\1f\11:\05\01\81\d0*\80\d6+\04\01\81\e0\80\f7)L\04\0a\04\02\83\11DL=\80\c2<\06\01\04U\05\1b4\02\81\0e,\04d\0cV\0a\80\ae8\1d\0d,\04\09\07\02\0e\06\80\9a\83\d8\04\11\03\0d\03w\04_\06\0c\04\01\0f\0c\048\08\0a\06(\08,\04\02>\81T\0c\1d\03\0a\058\07\1c\06\09\07\80\fa\84\06\00\01\03\05\05\06\06\02\07\06\08\07\09\11\0a\1c\0b\19\0c\1a\0d\10\0e\0c\0f\04\10\03\12\12\13\09\16\01\17\04\18\01\19\03\1a\07\1b\01\1c\02\1f\16 \03+\03-\0b.\010\041\022\01\a7\04\a9\02\aa\04\ab\08\fa\02\fb\05\fd\02\fe\03\ff\09\adxy\8b\8d\a20WX\8b\8c\90\1c\dd\0e\0fKL\fb\fc./?\5c]_\e2\84\8d\8e\91\92\a9\b1\ba\bb\c5\c6\c9\ca\de\e4\e5\ff\00\04\11\12)147:;=IJ]\84\8e\92\a9\b1\b4\ba\bb\c6\ca\ce\cf\e4\e5\00\04\0d\0e\11\12)14:;EFIJ^de\84\91\9b\9d\c9\ce\cf\0d\11):;EIW[\5c^_de\8d\91\a9\b4\ba\bb\c5\c9\df\e4\e5\f0\0d\11EIde\80\84\b2\bc\be\bf\d5\d7\f0\f1\83\85\8b\a4\a6\be\bf\c5\c7\cf\da\dbH\98\bd\cd\c6\ce\cfINOWY^_\89\8e\8f\b1\b6\b7\bf\c1\c6\c7\d7\11\16\17[\5c\f6\f7\fe\ff\80mq\de\df\0e\1fno\1c\1d_}~\ae\afM\bb\bc\16\17\1e\1fFGNOXZ\5c^~\7f\b5\c5\d4\d5\dc\f0\f1\f5rs\8ftu\96&./\a7\af\b7\bf\c7\cf\d7\df\9a\00@\97\980\8f\1f\ce\cf\d2\d4\ce\ffNOZ[\07\08\0f\10'/\ee\efno7=?BE\90\91Sgu\c8\c9\d0\d1\d8\d9\e7\fe\ff\00 _\22\82\df\04\82D\08\1b\04\06\11\81\ac\0e\80\ab\05\1f\08\81\1c\03\19\08\01\04/\044\04\07\03\01\07\06\07\11\0aP\0f\12\07U\07\03\04\1c\0a\09\03\08\03\07\03\02\03\03\03\0c\04\05\03\0b\06\01\0e\15\05N\07\1b\07W\07\02\06\17\0cP\04C\03-\03\01\04\11\06\0f\0c:\04\1d%_ m\04j%\80\c8\05\82\b0\03\1a\06\82\fd\03Y\07\16\09\18\09\14\0c\14\0cj\06\0a\06\1a\06Y\07+\05F\0a,\04\0c\04\01\031\0b,\04\1a\06\0b\03\80\ac\06\0a\06/1\80\f4\08<\03\0f\03>\058\08+\05\82\ff\11\18\08/\11-\03!\0f!\0f\80\8c\04\82\9a\16\0b\15\88\94\05/\05;\07\02\0e\18\09\80\be\22t\0c\80\d6\1a\81\10\05\80\e1\09\f2\9e\037\09\81\5c\14\80\b8\08\80\dd\15;\03\0a\068\08F\08\0c\06t\0b\1e\03Z\04Y\09\80\83\18\1c\0a\16\09L\04\80\8a\06\ab\a4\0c\17\041\a1\04\81\da&\07\0c\05\05\80\a6\10\81\f5\07\01 *\06L\04\80\8d\04\80\be\03\1b\03\0f\0dcore/src/unicode/unicode_data.rs\00\00\00u$\10\00 \00\00\00N\00\00\00(\00\00\00u$\10\00 \00\00\00Z\00\00\00\16\00\00\00 out of range for slice of length range end index \00\00\da$\10\00\10\00\00\00\b8$\10\00\22\00\00\00slice index starts at  but ends at \00\fc$\10\00\16\00\00\00\12%\10\00\0d\00\00\00\00\03\00\00\83\04 \00\91\05`\00]\13\a0\00\12\17 \1f\0c `\1f\ef, +*0\a0+o\a6`,\02\a8\e0,\1e\fb\e0-\00\fe 6\9e\ff`6\fd\01\e16\01\0a!7$\0d\e17\ab\0ea9/\18\e190\1c\e1J\f3\1e\e1N@4\a1R\1ea\e1S\f0jaTOo\e1T\9d\bcaU\00\cfaVe\d1\a1V\00\da!W\00\e0\a1X\ae\e2!Z\ec\e4\e1[\d0\e8a\5c \00\ee\5c\f0\01\7f]\00p\00\07\00-\01\01\01\02\01\02\01\01H\0b0\15\10\01e\07\02\06\02\02\01\04#\01\1e\1b[\0b:\09\09\01\18\04\01\09\01\03\01\05+\03;\09*\18\01 7\01\01\01\04\08\04\01\03\07\0a\02\1d\01:\01\01\01\02\04\08\01\09\01\0a\02\1a\01\02\029\01\04\02\04\02\02\03\03\01\1e\02\03\01\0b\029\01\04\05\01\02\04\01\14\02\16\06\01\01:\01\01\02\01\04\08\01\07\03\0a\02\1e\01;\01\01\01\0c\01\09\01(\01\03\017\01\01\03\05\03\01\04\07\02\0b\02\1d\01:\01\02\02\01\01\03\03\01\04\07\02\0b\02\1c\029\02\01\01\02\04\08\01\09\01\0a\02\1d\01H\01\04\01\02\03\01\01\08\01Q\01\02\07\0c\08b\01\02\09\0b\07I\02\1b\01\01\01\01\017\0e\01\05\01\02\05\0b\01$\09\01f\04\01\06\01\02\02\02\19\02\04\03\10\04\0d\01\02\02\06\01\0f\01\00\03\00\04\1c\03\1d\02\1e\02@\02\01\07\08\01\02\0b\09\01-\03\01\01u\02\22\01v\03\04\02\09\01\06\03\db\02\02\01:\01\01\07\01\01\01\01\02\08\06\0a\02\010\1f1\040\0a\04\03&\09\0c\02 \04\02\068\01\01\02\03\01\01\058\08\02\02\98\03\01\0d\01\07\04\01\06\01\03\02\c6@\00\01\c3!\00\03\8d\01` \00\06i\02\00\04\01\0a \02P\02\00\01\03\01\04\01\19\02\05\01\97\02\1a\12\0d\01&\08\19\0b\01\01,\030\01\02\04\02\02\02\01$\01C\06\02\02\02\02\0c\01\08\01/\013\01\01\03\02\02\05\02\01\01*\02\08\01\ee\01\02\01\04\01\00\01\00\10\10\10\00\02\00\01\e2\01\95\05\00\03\01\02\05\04(\03\04\01\a5\02\00\04A\05\00\02O\04F\0b1\04{\016\0f)\01\02\02\0a\031\04\02\02\07\01=\03$\05\01\08>\01\0c\024\09\01\01\08\04\02\01_\03\02\04\06\01\02\01\9d\01\03\08\15\029\02\01\01\01\01\0c\01\09\01\0e\07\03\05C\01\02\06\01\01\02\01\01\03\04\03\01\01\0e\02U\08\02\03\01\01\17\01Q\01\02\06\01\01\02\01\01\02\01\02\eb\01\02\04\06\02\01\02\1b\02U\08\02\01\01\02j\01\01\01\02\08e\01\01\01\02\04\01\05\00\09\01\02\f5\01\0a\04\04\01\90\04\02\02\04\01 \0a(\06\02\04\08\01\09\06\02\03.\0d\01\02\00\07\01\06\01\01R\16\02\07\01\02\01\02z\06\03\01\01\02\01\07\01\01H\02\03\01\01\01\00\02\0b\024\05\05\03\17\01\00\01\06\0f\00\0c\03\03\00\05;\07\00\01?\04Q\01\0b\02\00\02\00.\02\17\00\05\03\06\08\08\02\07\1e\04\94\03\007\042\08\01\0e\01\16\05\01\0f\00\07\01\11\02\07\01\02\01\05d\01\a0\07\00\01=\04\00\04\fe\02\00\07m\07\00`\80\f0\00"))
