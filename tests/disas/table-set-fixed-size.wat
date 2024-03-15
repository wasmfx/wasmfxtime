;;! target = "x86_64"
;;! test = "optimize"
;;! flags = "-O opt-level=0"

;; Test basic code generation for table WebAssembly instructions on
;; non-resizeable tables. Use optimized but with `opt-level=0` to legalize away
;; macro instructions.

(module
  (table (export "table") 7 7 externref)
  (func (export "table.set.const") (param externref)
    i32.const 0
    local.get 0
    table.set 0)
  (func (export "table.set.var") (param i32 externref)
    local.get 0
    local.get 1
    table.set 0))

;; function u0:0(i64 vmctx, i64, r64) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+72
;;     sig0 = (i64 vmctx, i64) system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: r64):
;;                                     v3 -> v0
;;                                     v22 -> v0
;;                                     v25 -> v0
;; @0051                               v4 = load.i64 notrap aligned v3+8
;; @0052                               v5 = iconst.i32 0
;; @0056                               v6 = iconst.i32 7
;; @0056                               v7 = icmp uge v5, v6  ; v5 = 0, v6 = 7
;; @0056                               brif v7, block7, block8
;;
;;                                 block7 cold:
;; @0056                               trap table_oob
;;
;;                                 block8:
;; @0056                               v8 = uextend.i64 v5  ; v5 = 0
;; @0056                               v9 = load.i64 notrap aligned v25+72
;;                                     v26 = iconst.i64 3
;; @0056                               v10 = ishl v8, v26  ; v26 = 3
;; @0056                               v11 = iadd v9, v10
;; @0056                               v12 = icmp.i32 uge v5, v6  ; v5 = 0, v6 = 7
;; @0056                               v13 = select_spectre_guard v12, v9, v11
;; @0056                               v14 = load.i64 notrap aligned table v13
;; @0056                               store.r64 notrap aligned table v2, v13
;; @0056                               v15 = is_null.r64 v2
;; @0056                               brif v15, block3, block2
;;
;;                                 block2:
;; @0056                               v16 = load.i64 notrap aligned v2
;;                                     v27 = iconst.i64 1
;; @0056                               v17 = iadd v16, v27  ; v27 = 1
;; @0056                               store notrap aligned v17, v2
;; @0056                               jump block3
;;
;;                                 block3:
;;                                     v28 = iconst.i64 0
;; @0056                               v18 = icmp.i64 eq v14, v28  ; v28 = 0
;; @0056                               brif v18, block6, block4
;;
;;                                 block4:
;; @0056                               v19 = load.i64 notrap aligned v14
;;                                     v29 = iconst.i64 -1
;; @0056                               v20 = iadd v19, v29  ; v29 = -1
;; @0056                               store notrap aligned v20, v14
;;                                     v30 = iconst.i64 0
;; @0056                               v21 = icmp eq v20, v30  ; v30 = 0
;; @0056                               brif v21, block5, block6
;;
;;                                 block5:
;; @0056                               v23 = load.i64 notrap aligned readonly v22+56
;; @0056                               v24 = load.i64 notrap aligned readonly v23+392
;; @0056                               call_indirect sig0, v24(v22, v14)
;; @0056                               jump block6
;;
;;                                 block6:
;; @0058                               jump block1
;;
;;                                 block1:
;; @0058                               return
;; }
;;
;; function u0:1(i64 vmctx, i64, i32, r64) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+72
;;     sig0 = (i64 vmctx, i64) system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32, v3: r64):
;;                                     v4 -> v0
;;                                     v22 -> v0
;;                                     v25 -> v0
;; @005a                               v5 = load.i64 notrap aligned v4+8
;; @005f                               v6 = iconst.i32 7
;; @005f                               v7 = icmp uge v2, v6  ; v6 = 7
;; @005f                               brif v7, block7, block8
;;
;;                                 block7 cold:
;; @005f                               trap table_oob
;;
;;                                 block8:
;; @005f                               v8 = uextend.i64 v2
;; @005f                               v9 = load.i64 notrap aligned v25+72
;;                                     v26 = iconst.i64 3
;; @005f                               v10 = ishl v8, v26  ; v26 = 3
;; @005f                               v11 = iadd v9, v10
;; @005f                               v12 = icmp.i32 uge v2, v6  ; v6 = 7
;; @005f                               v13 = select_spectre_guard v12, v9, v11
;; @005f                               v14 = load.i64 notrap aligned table v13
;; @005f                               store.r64 notrap aligned table v3, v13
;; @005f                               v15 = is_null.r64 v3
;; @005f                               brif v15, block3, block2
;;
;;                                 block2:
;; @005f                               v16 = load.i64 notrap aligned v3
;;                                     v27 = iconst.i64 1
;; @005f                               v17 = iadd v16, v27  ; v27 = 1
;; @005f                               store notrap aligned v17, v3
;; @005f                               jump block3
;;
;;                                 block3:
;;                                     v28 = iconst.i64 0
;; @005f                               v18 = icmp.i64 eq v14, v28  ; v28 = 0
;; @005f                               brif v18, block6, block4
;;
;;                                 block4:
;; @005f                               v19 = load.i64 notrap aligned v14
;;                                     v29 = iconst.i64 -1
;; @005f                               v20 = iadd v19, v29  ; v29 = -1
;; @005f                               store notrap aligned v20, v14
;;                                     v30 = iconst.i64 0
;; @005f                               v21 = icmp eq v20, v30  ; v30 = 0
;; @005f                               brif v21, block5, block6
;;
;;                                 block5:
;; @005f                               v23 = load.i64 notrap aligned readonly v22+56
;; @005f                               v24 = load.i64 notrap aligned readonly v23+392
;; @005f                               call_indirect sig0, v24(v22, v14)
;; @005f                               jump block6
;;
;;                                 block6:
;; @0061                               jump block1
;;
;;                                 block1:
;; @0061                               return
;; }
