;;! target = "x86_64"
;;! test = "optimize"
;;! flags = "-O opt-level=0"

;; Test basic code generation for table WebAssembly instructions.
;; Use optimization but with `opt-level=0` to legalize away table_addr instructions.

(module
  (table (export "table") 1 externref)
  (func (export "table.get.const") (result externref)
    i32.const 0
    table.get 0)
  (func (export "table.get.var") (param i32) (result externref)
    local.get 0
    table.get 0))

;; function u0:0(i64 vmctx, i64) -> r64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+72
;;     gv5 = load.i32 notrap aligned gv3+80
;;     sig0 = (i64 vmctx, r64) system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;;                                     v3 -> v0
;;                                     v16 -> v0
;;                                     v21 -> v0
;;                                     v27 -> v0
;;                                     v28 -> v0
;; @0050                               v4 = load.i64 notrap aligned v3+8
;; @0051                               v5 = iconst.i32 0
;; @0053                               v6 = load.i32 notrap aligned v27+80
;; @0053                               v7 = icmp uge v5, v6  ; v5 = 0
;; @0053                               brif v7, block6, block7
;;
;;                                 block6 cold:
;; @0053                               trap table_oob
;;
;;                                 block7:
;; @0053                               v8 = uextend.i64 v5  ; v5 = 0
;; @0053                               v9 = load.i64 notrap aligned v28+72
;;                                     v29 = iconst.i64 3
;; @0053                               v10 = ishl v8, v29  ; v29 = 3
;; @0053                               v11 = iadd v9, v10
;; @0053                               v12 = icmp.i32 uge v5, v6  ; v5 = 0
;; @0053                               v13 = select_spectre_guard v12, v9, v11
;; @0053                               v14 = load.r64 notrap aligned table v13
;;                                     v2 -> v14
;; @0053                               v15 = is_null v14
;; @0053                               brif v15, block2, block3
;;
;;                                 block3:
;; @0053                               v17 = load.i64 notrap aligned v16+32
;; @0053                               v18 = load.i64 notrap aligned v17
;; @0053                               v19 = load.i64 notrap aligned v17+8
;; @0053                               v20 = icmp eq v18, v19
;; @0053                               brif v20, block4, block5
;;
;;                                 block5:
;; @0053                               v24 = load.i64 notrap aligned v14
;;                                     v30 = iconst.i64 1
;; @0053                               v25 = iadd v24, v30  ; v30 = 1
;; @0053                               store notrap aligned v25, v14
;; @0053                               store.r64 notrap aligned v14, v18
;;                                     v31 = iconst.i64 8
;; @0053                               v26 = iadd.i64 v18, v31  ; v31 = 8
;; @0053                               store notrap aligned v26, v17
;; @0053                               jump block2
;;
;;                                 block4:
;; @0053                               v22 = load.i64 notrap aligned readonly v21+56
;; @0053                               v23 = load.i64 notrap aligned readonly v22+400
;; @0053                               call_indirect sig0, v23(v21, v14)
;; @0053                               jump block2
;;
;;                                 block2:
;; @0055                               jump block1
;;
;;                                 block1:
;; @0055                               return v2
;; }
;;
;; function u0:1(i64 vmctx, i64, i32) -> r64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+72
;;     gv5 = load.i32 notrap aligned gv3+80
;;     sig0 = (i64 vmctx, r64) system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32):
;;                                     v4 -> v0
;;                                     v16 -> v0
;;                                     v21 -> v0
;;                                     v27 -> v0
;;                                     v28 -> v0
;; @0057                               v5 = load.i64 notrap aligned v4+8
;; @005a                               v6 = load.i32 notrap aligned v27+80
;; @005a                               v7 = icmp uge v2, v6
;; @005a                               brif v7, block6, block7
;;
;;                                 block6 cold:
;; @005a                               trap table_oob
;;
;;                                 block7:
;; @005a                               v8 = uextend.i64 v2
;; @005a                               v9 = load.i64 notrap aligned v28+72
;;                                     v29 = iconst.i64 3
;; @005a                               v10 = ishl v8, v29  ; v29 = 3
;; @005a                               v11 = iadd v9, v10
;; @005a                               v12 = icmp.i32 uge v2, v6
;; @005a                               v13 = select_spectre_guard v12, v9, v11
;; @005a                               v14 = load.r64 notrap aligned table v13
;;                                     v3 -> v14
;; @005a                               v15 = is_null v14
;; @005a                               brif v15, block2, block3
;;
;;                                 block3:
;; @005a                               v17 = load.i64 notrap aligned v16+32
;; @005a                               v18 = load.i64 notrap aligned v17
;; @005a                               v19 = load.i64 notrap aligned v17+8
;; @005a                               v20 = icmp eq v18, v19
;; @005a                               brif v20, block4, block5
;;
;;                                 block5:
;; @005a                               v24 = load.i64 notrap aligned v14
;;                                     v30 = iconst.i64 1
;; @005a                               v25 = iadd v24, v30  ; v30 = 1
;; @005a                               store notrap aligned v25, v14
;; @005a                               store.r64 notrap aligned v14, v18
;;                                     v31 = iconst.i64 8
;; @005a                               v26 = iadd.i64 v18, v31  ; v31 = 8
;; @005a                               store notrap aligned v26, v17
;; @005a                               jump block2
;;
;;                                 block4:
;; @005a                               v22 = load.i64 notrap aligned readonly v21+56
;; @005a                               v23 = load.i64 notrap aligned readonly v22+400
;; @005a                               call_indirect sig0, v23(v21, v14)
;; @005a                               jump block2
;;
;;                                 block2:
;; @005c                               jump block1
;;
;;                                 block1:
;; @005c                               return v3
;; }
