;;! target = "x86_64"
;;! flags = [
;;!   "-Ccranelift-enable-heap-access-spectre-mitigation=false",
;;!   "-Ostatic-memory-maximum-size=0",
;;!   "-Odynamic-memory-guard-size=0",
;;! ]

;; Test that dynamic memories with `min_size == max_size` don't actually load
;; their dynamic memory bound, since it is a constant.

(module
  (memory 1 1)

  (func (export "do_store") (param i32 i32)
    local.get 0
    local.get 1
    i32.store8 offset=0)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load8_u offset=0))

;; function u0:0(i64 vmctx, i64, i32, i32) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+88
;;     gv5 = load.i64 notrap aligned checked gv3+80
;;     sig0 = (i64 vmctx, i32 uext, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32, v3: i32):
;; @003c                               v4 = global_value.i64 gv3
;; @003c                               v5 = load.i64 notrap aligned v4+8
;; @0041                               v6 = uextend.i64 v2
;; @0041                               v7 = iconst.i64 0x0001_0000
;; @0041                               v8 = icmp uge v6, v7  ; v7 = 0x0001_0000
;; @0041                               trapnz v8, heap_oob
;; @0041                               v9 = global_value.i64 gv5
;; @0041                               v10 = iadd v9, v6
;; @0041                               istore8 little heap v3, v10
;; @0044                               jump block1
;;
;;                                 block1:
;; @0044                               return
;; }
;;
;; function u0:1(i64 vmctx, i64, i32) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+88
;;     gv5 = load.i64 notrap aligned checked gv3+80
;;     sig0 = (i64 vmctx, i32 uext, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32):
;; @0046                               v4 = global_value.i64 gv3
;; @0046                               v5 = load.i64 notrap aligned v4+8
;; @0049                               v6 = uextend.i64 v2
;; @0049                               v7 = iconst.i64 0x0001_0000
;; @0049                               v8 = icmp uge v6, v7  ; v7 = 0x0001_0000
;; @0049                               trapnz v8, heap_oob
;; @0049                               v9 = global_value.i64 gv5
;; @0049                               v10 = iadd v9, v6
;; @0049                               v11 = uload8.i32 little heap v10
;; @004c                               jump block1(v11)
;;
;;                                 block1(v3: i32):
;; @004c                               return v3
;; }
