;;! target = "x86_64"

(module
  (func (export "multiLoop") (param i64 i64) (result i64 i64)
    (local.get 1)
    (local.get 0)
    (loop (param i64 i64) (result i64 i64)
       return)))

;; function u0:0(i64 vmctx, i64, i64, i64) -> i64, i64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i64, v3: i64):
;; @0029                               v6 = global_value.i64 gv3
;; @0029                               v7 = load.i64 notrap aligned v6+8
;; @002e                               jump block2(v3, v2)
;;
;;                                 block2(v8: i64, v9: i64):
;; @0030                               return v8, v9
;; }
