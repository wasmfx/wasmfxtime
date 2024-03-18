;;! target = "x86_64"

(module
  (func (param i32)
    (loop
      (block
        local.get 0
        br_if 0
        br 1)))

  (func (param i32)
    (loop
      (block
        br 1
        call $empty)))

  (func $empty)

  (func (param i32) (result i32)
    i32.const 1
    return
    i32.const 42)
)
;; function u0:0(i64 vmctx, i64, i32) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32):
;;                                     v5 -> v2
;; @0022                               v3 = global_value.i64 gv3
;; @0022                               v4 = load.i64 notrap aligned v3+8
;; @0023                               jump block2
;;
;;                                 block2:
;; @0029                               brif.i32 v5, block4, block5
;;
;;                                 block5:
;; @002b                               jump block2
;;
;;                                 block4:
;; @002e                               jump block3
;;
;;                                 block3:
;; @002f                               jump block1
;;
;;                                 block1:
;; @002f                               return
;; }
;;
;; function u0:1(i64 vmctx, i64, i32) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32):
;; @0031                               v3 = global_value.i64 gv3
;; @0031                               v4 = load.i64 notrap aligned v3+8
;; @0032                               jump block2
;;
;;                                 block2:
;; @0036                               jump block2
;; }
;;
;; function u0:2(i64 vmctx, i64) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @003e                               v2 = global_value.i64 gv3
;; @003e                               v3 = load.i64 notrap aligned v2+8
;; @003f                               jump block1
;;
;;                                 block1:
;; @003f                               return
;; }
;;
;; function u0:3(i64 vmctx, i64, i32) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32):
;; @0041                               v4 = global_value.i64 gv3
;; @0041                               v5 = load.i64 notrap aligned v4+8
;; @0042                               v6 = iconst.i32 1
;; @0044                               return v6  ; v6 = 1
;; }
