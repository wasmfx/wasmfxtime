;;! target = "x86_64"
;;! test = "optimize"
;;! flags = "-O opt-level=0"

;; Test basic code generation for table WebAssembly instructions on
;; non-resizeable tables. Use optimized but with `opt-level=0` to legalize away
;; macro instructions.

(module
  (table (export "table") 7 7 externref)
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
;; @0051                               v4 = load.i64 notrap aligned v3+8
;; @0052                               v5 = iconst.i32 0
;; @0054                               v6 = iconst.i32 7
;; @0054                               v7 = icmp uge v5, v6  ; v5 = 0, v6 = 7
;; @0054                               v8 = uextend.i64 v5  ; v5 = 0
;; @0054                               v9 = load.i64 notrap aligned v27+72
;;                                     v28 = iconst.i64 3
;; @0054                               v10 = ishl v8, v28  ; v28 = 3
;; @0054                               v11 = iadd v9, v10
;; @0054                               v12 = iconst.i64 0
;; @0054                               v13 = select_spectre_guard v7, v12, v11  ; v12 = 0
;; @0054                               v14 = load.r64 table_oob aligned table v13
;;                                     v2 -> v14
;; @0054                               v15 = is_null v14
;; @0054                               brif v15, block2, block3
;;
;;                                 block3:
;; @0054                               v17 = load.i64 notrap aligned v16+32
;; @0054                               v18 = load.i64 notrap aligned v17
;; @0054                               v19 = load.i64 notrap aligned v17+8
;; @0054                               v20 = icmp eq v18, v19
;; @0054                               brif v20, block4, block5
;;
;;                                 block5:
;; @0054                               v24 = load.i64 notrap aligned v14
;;                                     v29 = iconst.i64 1
;; @0054                               v25 = iadd v24, v29  ; v29 = 1
;; @0054                               store notrap aligned v25, v14
;; @0054                               store.r64 notrap aligned v14, v18
;;                                     v30 = iconst.i64 8
;; @0054                               v26 = iadd.i64 v18, v30  ; v30 = 8
;; @0054                               store notrap aligned v26, v17
;; @0054                               jump block2
;;
;;                                 block4:
;; @0054                               v22 = load.i64 notrap aligned readonly v21+56
;; @0054                               v23 = load.i64 notrap aligned readonly v22+400
;; @0054                               call_indirect sig0, v23(v21, v14)
;; @0054                               jump block2
;;
;;                                 block2:
;; @0056                               jump block1
;;
;;                                 block1:
;; @0056                               return v2
;; }
;;
;; function u0:1(i64 vmctx, i64, i32) -> r64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+72
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
;; @0058                               v5 = load.i64 notrap aligned v4+8
;; @005b                               v6 = iconst.i32 7
;; @005b                               v7 = icmp uge v2, v6  ; v6 = 7
;; @005b                               v8 = uextend.i64 v2
;; @005b                               v9 = load.i64 notrap aligned v27+72
;;                                     v28 = iconst.i64 3
;; @005b                               v10 = ishl v8, v28  ; v28 = 3
;; @005b                               v11 = iadd v9, v10
;; @005b                               v12 = iconst.i64 0
;; @005b                               v13 = select_spectre_guard v7, v12, v11  ; v12 = 0
;; @005b                               v14 = load.r64 table_oob aligned table v13
;;                                     v3 -> v14
;; @005b                               v15 = is_null v14
;; @005b                               brif v15, block2, block3
;;
;;                                 block3:
;; @005b                               v17 = load.i64 notrap aligned v16+32
;; @005b                               v18 = load.i64 notrap aligned v17
;; @005b                               v19 = load.i64 notrap aligned v17+8
;; @005b                               v20 = icmp eq v18, v19
;; @005b                               brif v20, block4, block5
;;
;;                                 block5:
;; @005b                               v24 = load.i64 notrap aligned v14
;;                                     v29 = iconst.i64 1
;; @005b                               v25 = iadd v24, v29  ; v29 = 1
;; @005b                               store notrap aligned v25, v14
;; @005b                               store.r64 notrap aligned v14, v18
;;                                     v30 = iconst.i64 8
;; @005b                               v26 = iadd.i64 v18, v30  ; v30 = 8
;; @005b                               store notrap aligned v26, v17
;; @005b                               jump block2
;;
;;                                 block4:
;; @005b                               v22 = load.i64 notrap aligned readonly v21+56
;; @005b                               v23 = load.i64 notrap aligned readonly v22+400
;; @005b                               call_indirect sig0, v23(v21, v14)
;; @005b                               jump block2
;;
;;                                 block2:
;; @005d                               jump block1
;;
;;                                 block1:
;; @005d                               return v3
;; }
