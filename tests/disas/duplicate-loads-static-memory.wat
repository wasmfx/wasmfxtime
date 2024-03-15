;;! target = "x86_64"
;;! test = "optimize"

(module
  (memory (export "memory") 1)
  (func (export "load-without-offset") (param i32) (result i32 i32)
    local.get 0
    i32.load
    local.get 0
    i32.load
  )
  (func (export "load-with-offset") (param i32) (result i32 i32)
    local.get 0
    i32.load offset=1234
    local.get 0
    i32.load offset=1234
  )
)

;; function u0:0(i64 vmctx, i64, i32) -> i32, i32 fast {
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
;;                                     v5 -> v0
;;                                     v15 -> v0
;;                                     v16 -> v0
;; @0057                               v8 = load.i64 notrap aligned readonly checked v0+80
;; @0057                               v7 = uextend.i64 v2
;; @0057                               v9 = iadd v8, v7
;; @0057                               v10 = load.i32 little heap v9
;;                                     v3 -> v10
;; @005f                               jump block1
;;
;;                                 block1:
;; @005f                               return v10, v10
;; }
;;
;; function u0:1(i64 vmctx, i64, i32) -> i32, i32 fast {
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
;;                                     v5 -> v0
;;                                     v19 -> v0
;;                                     v20 -> v0
;; @0064                               v8 = load.i64 notrap aligned readonly checked v0+80
;; @0064                               v7 = uextend.i64 v2
;; @0064                               v9 = iadd v8, v7
;; @0064                               v10 = iconst.i64 1234
;; @0064                               v11 = iadd v9, v10  ; v10 = 1234
;; @0064                               v12 = load.i32 little heap v11
;;                                     v3 -> v12
;; @006e                               jump block1
;;
;;                                 block1:
;; @006e                               return v12, v12
;; }
