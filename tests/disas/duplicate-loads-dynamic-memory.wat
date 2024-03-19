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
;;                                     v21 -> v0
;;                                     v22 -> v0
;;                                     v23 -> v0
;;                                     v24 -> v0
;; @0057                               v6 = load.i64 notrap aligned v0+88
;; @0057                               v8 = load.i64 notrap aligned checked v0+80
;; @0057                               v5 = uextend.i64 v2
;; @0057                               v7 = icmp ugt v5, v6
;; @0057                               v10 = iconst.i64 0
;; @0057                               v9 = iadd v8, v5
;; @0057                               v11 = select_spectre_guard v7, v10, v9  ; v10 = 0
;; @0057                               v12 = load.i32 little heap v11
;;                                     v3 -> v12
;; @005f                               jump block1
;;
;;                                 block1:
;; @005f                               return v12, v12
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
;;                                     v25 -> v0
;;                                     v26 -> v0
;;                                     v27 -> v0
;;                                     v28 -> v0
;; @0064                               v6 = load.i64 notrap aligned v0+88
;; @0064                               v8 = load.i64 notrap aligned checked v0+80
;; @0064                               v5 = uextend.i64 v2
;; @0064                               v7 = icmp ugt v5, v6
;; @0064                               v12 = iconst.i64 0
;; @0064                               v9 = iadd v8, v5
;; @0064                               v10 = iconst.i64 1234
;; @0064                               v11 = iadd v9, v10  ; v10 = 1234
;; @0064                               v13 = select_spectre_guard v7, v12, v11  ; v12 = 0
;; @0064                               v14 = load.i32 little heap v13
;;                                     v3 -> v14
;; @006e                               jump block1
;;
;;                                 block1:
;; @006e                               return v14, v14
;; }
