;;! target = "x86_64"

(module
  (memory 1)
  (func (export "f32.load") (param i32) (result f32)
    local.get 0
    f32.load))

;; function u0:0(i64 vmctx, i64, i32) -> f32 fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i32):
;; @002b                               v4 = global_value.i64 gv3
;; @002b                               v5 = load.i64 notrap aligned v4+8
;; @002e                               v6 = uextend.i64 v2
;; @002e                               v7 = global_value.i64 gv4
;; @002e                               v8 = iadd v7, v6
;; @002e                               v9 = load.f32 little heap v8
;; @0031                               jump block1(v9)
;;
;;                                 block1(v3: f32):
;; @0031                               return v3
;; }
