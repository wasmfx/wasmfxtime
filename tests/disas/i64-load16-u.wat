;;! target = "x86_64"

;; Test basic code generation for i64 memory WebAssembly instructions.

(module
  (memory 1)
  (func (export "i64.load16_u") (param i32) (result i64)
    local.get 0
    i64.load16_u))

;; function u0:0(i64 vmctx, i64, i32) -> i64 fast {
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
;; @0032                               v4 = uextend.i64 v2
;; @0032                               v5 = global_value.i64 gv4
;; @0032                               v6 = iadd v5, v4
;; @0032                               v7 = uload16.i64 little heap v6
;; @0035                               jump block1(v7)
;;
;;                                 block1(v3: i64):
;; @0035                               return v3
;; }
