;;! target = "x86_64"

(module
  (global $x (mut i32) (i32.const 4))
  (memory 1)
  (func $main (local i32)
    (i32.store (i32.const 0) (global.get $x))
  )
  (start $main)
)

;; function u0:0(i64 vmctx, i64) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned readonly checked gv3+80
;;     sig0 = (i64 vmctx, i32 uext, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @0027                               v2 = iconst.i32 0
;; @0027                               v3 = global_value.i64 gv3
;; @0027                               v4 = load.i64 notrap aligned v3+8
;; @0029                               v5 = iconst.i32 0
;; @002b                               v6 = global_value.i64 gv3
;; @002b                               v7 = load.i32 notrap aligned table v6+96
;; @002d                               v8 = uextend.i64 v5  ; v5 = 0
;; @002d                               v9 = global_value.i64 gv4
;; @002d                               v10 = iadd v9, v8
;; @002d                               store little heap v7, v10
;; @0030                               jump block1
;;
;;                                 block1:
;; @0030                               return
;; }
