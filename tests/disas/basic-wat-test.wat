;;! target = "x86_64"

(module
  (memory 0)
  (func (param i32 i32) (result i32)
    local.get 0
    i32.load
    local.get 1
    i32.load
    i32.add))

;; function u0:0(i64 vmctx, i64, i32, i32) -> i32 fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i32, v3: i32):
;; @001e                               v5 = global_value.i64 gv3
;; @001e                               v6 = load.i64 notrap aligned v5+8
;; @0021                               v7 = uextend.i64 v2
;; @0021                               v8 = global_value.i64 gv4
;; @0021                               v9 = iadd v8, v7
;; @0021                               v10 = load.i32 little heap v9
;; @0026                               v11 = uextend.i64 v3
;; @0026                               v12 = global_value.i64 gv4
;; @0026                               v13 = iadd v12, v11
;; @0026                               v14 = load.i32 little heap v13
;; @0029                               v15 = iadd v10, v14
;; @002a                               jump block1(v15)
;;
;;                                 block1(v4: i32):
;; @002a                               return v4
;; }
