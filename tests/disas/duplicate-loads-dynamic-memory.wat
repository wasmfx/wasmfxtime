;;! target = "x86_64"
;;! test = "optimize"
;;! flags = [
;;!   "-Ccranelift-enable-heap-access-spectre-mitigation",
;;!   "-Oopt-level=s",
;;!   "-Ostatic-memory-maximum-size=0",
;;! ]

(module
  (memory (export "memory") 0)
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
;;     gv4 = load.i64 notrap aligned gv3+88
;;     gv5 = load.i64 notrap aligned checked gv3+80
;;     sig0 = (i64 vmctx, i32 uext, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32):
;;                                     v5 -> v0
;;                                     v23 -> v0
;;                                     v24 -> v0
;;                                     v25 -> v0
;;                                     v26 -> v0
;; @0057                               v8 = load.i64 notrap aligned v0+88
;; @0057                               v10 = load.i64 notrap aligned checked v0+80
;; @0057                               v7 = uextend.i64 v2
;; @0057                               v9 = icmp ugt v7, v8
;; @0057                               v12 = iconst.i64 0
;; @0057                               v11 = iadd v10, v7
;; @0057                               v13 = select_spectre_guard v9, v12, v11  ; v12 = 0
;; @0057                               v14 = load.i32 little heap v13
;;                                     v3 -> v14
;; @005f                               jump block1
;;
;;                                 block1:
;; @005f                               return v14, v14
;; }
;;
;; function u0:1(i64 vmctx, i64, i32) -> i32, i32 fast {
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
;;                                     v5 -> v0
;;                                     v27 -> v0
;;                                     v28 -> v0
;;                                     v29 -> v0
;;                                     v30 -> v0
;; @0064                               v8 = load.i64 notrap aligned v0+88
;; @0064                               v10 = load.i64 notrap aligned checked v0+80
;; @0064                               v7 = uextend.i64 v2
;; @0064                               v9 = icmp ugt v7, v8
;; @0064                               v14 = iconst.i64 0
;; @0064                               v11 = iadd v10, v7
;; @0064                               v12 = iconst.i64 1234
;; @0064                               v13 = iadd v11, v12  ; v12 = 1234
;; @0064                               v15 = select_spectre_guard v9, v14, v13  ; v14 = 0
;; @0064                               v16 = load.i32 little heap v15
;;                                     v3 -> v16
;; @006e                               jump block1
;;
;;                                 block1:
;; @006e                               return v16, v16
;; }
