;;! target = "x86_64"

(module
  (func $imported (import "env" "f") (param i32) (result i32))
  (func $local (result externref externref funcref funcref)
    global.get 0
    global.get 1
    global.get 2
    global.get 3)

  (global (export "externref-imported") externref (ref.null extern))
  (global (export "externref-local") externref (ref.null extern))
  (global (export "funcref-imported") funcref (ref.func $imported))
  (global (export "funcref-local") funcref (ref.func $local)))

;; function u0:1(i64 vmctx, i64) -> r64, r64, i64, i64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext) -> r64 system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @008e                               v6 = global_value.i64 gv3
;; @008e                               v7 = load.i64 notrap aligned v6+8
;; @008f                               v8 = global_value.i64 gv3
;; @008f                               v9 = load.i64 notrap aligned readonly v8+56
;; @008f                               v10 = load.i64 notrap aligned readonly v9+408
;; @008f                               v11 = iconst.i32 0
;; @008f                               v12 = call_indirect sig0, v10(v8, v11)  ; v11 = 0
;; @0091                               v13 = global_value.i64 gv3
;; @0091                               v14 = load.i64 notrap aligned readonly v13+56
;; @0091                               v15 = load.i64 notrap aligned readonly v14+408
;; @0091                               v16 = iconst.i32 1
;; @0091                               v17 = call_indirect sig0, v15(v13, v16)  ; v16 = 1
;; @0093                               v18 = global_value.i64 gv3
;; @0093                               v19 = load.i64 notrap aligned table v18+144
;; @0095                               v20 = global_value.i64 gv3
;; @0095                               v21 = load.i64 notrap aligned table v20+160
;; @0097                               jump block1(v12, v17, v19, v21)
;;
;;                                 block1(v2: r64, v3: r64, v4: i64, v5: i64):
;; @0097                               return v2, v3, v4, v5
;; }
