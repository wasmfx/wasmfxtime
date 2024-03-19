;;! target = "x86_64"
;;! test = "optimize"
;;! flags = "-O opt-level=0"

;; Test basic code generation for table WebAssembly instructions.
;; Use optimization but with `opt-level=0` to legalize away table_addr instructions.

(module
  (table (export "table") 1 externref)
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
;;     gv5 = load.i32 notrap aligned gv3+80
;;     sig0 = (i64 vmctx, i64) system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: r64):
;;                                     v3 -> v0
;;                                     v22 -> v0
;;                                     v25 -> v0
;;                                     v26 -> v0
;; @0050                               v4 = load.i64 notrap aligned v3+8
;; @0051                               v5 = iconst.i32 0
;; @0055                               v6 = load.i32 notrap aligned v25+80
;; @0055                               v7 = icmp uge v5, v6  ; v5 = 0
;; @0055                               v8 = uextend.i64 v5  ; v5 = 0
;; @0055                               v9 = load.i64 notrap aligned v26+72
;;                                     v27 = iconst.i64 3
;; @0055                               v10 = ishl v8, v27  ; v27 = 3
;; @0055                               v11 = iadd v9, v10
;; @0055                               v12 = iconst.i64 0
;; @0055                               v13 = select_spectre_guard v7, v12, v11  ; v12 = 0
;; @0055                               v14 = load.i64 table_oob aligned table v13
;; @0055                               store notrap aligned table v2, v13
;; @0055                               v15 = is_null v2
;; @0055                               brif v15, block3, block2
;;
;;                                 block2:
;; @0055                               v16 = load.i64 notrap aligned v2
;;                                     v28 = iconst.i64 1
;; @0055                               v17 = iadd v16, v28  ; v28 = 1
;; @0055                               store notrap aligned v17, v2
;; @0055                               jump block3
;;
;;                                 block3:
;;                                     v29 = iconst.i64 0
;; @0055                               v18 = icmp.i64 eq v14, v29  ; v29 = 0
;; @0055                               brif v18, block6, block4
;;
;;                                 block4:
;; @0055                               v19 = load.i64 notrap aligned v14
;;                                     v30 = iconst.i64 -1
;; @0055                               v20 = iadd v19, v30  ; v30 = -1
;; @0055                               store notrap aligned v20, v14
;;                                     v31 = iconst.i64 0
;; @0055                               v21 = icmp eq v20, v31  ; v31 = 0
;; @0055                               brif v21, block5, block6
;;
;;                                 block5:
;; @0055                               v23 = load.i64 notrap aligned readonly v22+56
;; @0055                               v24 = load.i64 notrap aligned readonly v23+392
;; @0055                               call_indirect sig0, v24(v22, v14)
;; @0055                               jump block6
;;
;;                                 block6:
;; @0057                               jump block1
;;
;;                                 block1:
;; @0057                               return
;; }
;;
;; function u0:1(i64 vmctx, i64, i32, r64) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+72
;;     gv5 = load.i32 notrap aligned gv3+80
;;     sig0 = (i64 vmctx, i64) system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32, v3: r64):
;;                                     v4 -> v0
;;                                     v22 -> v0
;;                                     v25 -> v0
;;                                     v26 -> v0
;; @0059                               v5 = load.i64 notrap aligned v4+8
;; @005e                               v6 = load.i32 notrap aligned v25+80
;; @005e                               v7 = icmp uge v2, v6
;; @005e                               v8 = uextend.i64 v2
;; @005e                               v9 = load.i64 notrap aligned v26+72
;;                                     v27 = iconst.i64 3
;; @005e                               v10 = ishl v8, v27  ; v27 = 3
;; @005e                               v11 = iadd v9, v10
;; @005e                               v12 = iconst.i64 0
;; @005e                               v13 = select_spectre_guard v7, v12, v11  ; v12 = 0
;; @005e                               v14 = load.i64 table_oob aligned table v13
;; @005e                               store notrap aligned table v3, v13
;; @005e                               v15 = is_null v3
;; @005e                               brif v15, block3, block2
;;
;;                                 block2:
;; @005e                               v16 = load.i64 notrap aligned v3
;;                                     v28 = iconst.i64 1
;; @005e                               v17 = iadd v16, v28  ; v28 = 1
;; @005e                               store notrap aligned v17, v3
;; @005e                               jump block3
;;
;;                                 block3:
;;                                     v29 = iconst.i64 0
;; @005e                               v18 = icmp.i64 eq v14, v29  ; v29 = 0
;; @005e                               brif v18, block6, block4
;;
;;                                 block4:
;; @005e                               v19 = load.i64 notrap aligned v14
;;                                     v30 = iconst.i64 -1
;; @005e                               v20 = iadd v19, v30  ; v30 = -1
;; @005e                               store notrap aligned v20, v14
;;                                     v31 = iconst.i64 0
;; @005e                               v21 = icmp eq v20, v31  ; v31 = 0
;; @005e                               brif v21, block5, block6
;;
;;                                 block5:
;; @005e                               v23 = load.i64 notrap aligned readonly v22+56
;; @005e                               v24 = load.i64 notrap aligned readonly v23+392
;; @005e                               call_indirect sig0, v24(v22, v14)
;; @005e                               jump block6
;;
;;                                 block6:
;; @0060                               jump block1
;;
;;                                 block1:
;; @0060                               return
;; }
