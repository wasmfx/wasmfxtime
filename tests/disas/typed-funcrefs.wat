;;! target = "x86_64"
;;! test = "optimize"
;;! flags = [ "-Wfunction-references=y" ]

;; This test is meant to simulate how typed funcrefs in a table may be
;; used for ICs (inline caches) in a Wasm module compiled from a dynamic
;; language. In native JIT engines, IC chains have head pointers that
;; are raw code pointers and IC-using code can call each with a few ops
;; (load pointer, call indirect). We'd like similar efficiency by
;; storing funcrefs for the first IC in each chain in a typed-funcref
;; table.

(module
  (type $ic-stub (func (param i32 i32 i32 i32) (result i32)))

  ;; This syntax declares a table that is exactly 100 elements, whose
  ;; elements are non-nullable function references, and whose default
  ;; value (needed because non-nullable) is a pointer to `$ic1`.
  (table $ic-sites 100 100 (ref $ic-stub) (ref.func $ic1))

  (func $ic1 (param i32 i32 i32 i32) (result i32)
        local.get 0)

  (func $call-ics (param i32 i32 i32 i32) (result i32)
        (local $sum i32)

        ;; IC callsite index 1 (arbitrary).
        local.get 0
        local.get 1
        local.get 2
        local.get 3
        i32.const 1
        table.get $ic-sites
        call_ref $ic-stub
        local.get $sum
        i32.add
        local.set $sum

        ;; IC callsite index 2 (arbitrary).
        local.get 0
        local.get 1
        local.get 2
        local.get 3
        i32.const 2
        table.get $ic-sites
        call_ref $ic-stub
        local.get $sum
        i32.add
        local.set $sum

        local.get $sum))
;; function u0:0(i64 vmctx, i64, i32, i32, i32, i32) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32, v3: i32, v4: i32, v5: i32):
;;                                     v7 -> v0
;;                                     v6 -> v2
;; @002c                               jump block1
;;
;;                                 block1:
;; @002c                               return v2
;; }
;;
;; function u0:1(i64 vmctx, i64, i32, i32, i32, i32) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+72
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i64 system_v
;;     sig1 = (i64 vmctx, i64, i32, i32, i32, i32) -> i32 fast
;;     sig2 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig3 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32, v3: i32, v4: i32, v5: i32):
;;                                     v8 -> v0
;;                                     v23 -> v0
;;                                     v49 -> v0
;;                                     v58 -> v0
;;                                     v61 -> v0
;;                                     v32 -> v2
;;                                     v33 -> v3
;;                                     v34 -> v4
;;                                     v35 -> v5
;;                                     v64 = iconst.i8 0
;; @003b                               brif v64, block6, block7  ; v64 = 0
;;
;;                                 block6 cold:
;; @003b                               trap table_oob
;;
;;                                 block7:
;; @003b                               v14 = load.i64 notrap aligned v0+72
;;                                     v81 = iconst.i8 0
;;                                     v72 = iconst.i64 8
;; @003b                               v16 = iadd v14, v72  ; v72 = 8
;; @003b                               v18 = select_spectre_guard v81, v14, v16  ; v81 = 0
;; @003b                               v19 = load.i64 notrap aligned table v18
;;                                     v60 = iconst.i64 -2
;; @003b                               v20 = band v19, v60  ; v60 = -2
;; @003b                               brif v19, block3(v20), block2
;;
;;                                 block2 cold:
;; @004e                               v50 = load.i64 notrap aligned readonly v0+56
;; @004e                               v51 = load.i64 notrap aligned readonly v50+72
;; @002f                               v7 = iconst.i32 0
;;                                     v30 -> v7
;; @0039                               v10 = iconst.i32 1
;; @003b                               v26 = call_indirect sig0, v51(v0, v7, v10)  ; v7 = 0, v10 = 1
;; @003b                               jump block3(v26)
;;
;;                                 block3(v21: i64):
;; @003d                               brif v21, block9, block8
;;
;;                                 block8 cold:
;; @003d                               trap null_reference
;;
;;                                 block9:
;; @003d                               v27 = load.i64 notrap aligned readonly v21+16
;; @003d                               v28 = load.i64 notrap aligned readonly v21+32
;; @003d                               v29 = call_indirect sig1, v27(v28, v0, v2, v3, v4, v5)
;;                                     v82 = iconst.i8 0
;; @004e                               brif v82, block10, block11  ; v82 = 0
;;
;;                                 block10 cold:
;; @004e                               trap table_oob
;;
;;                                 block11:
;; @004e                               v40 = load.i64 notrap aligned v0+72
;;                                     v83 = iconst.i8 0
;;                                     v80 = iconst.i64 16
;; @004e                               v42 = iadd v40, v80  ; v80 = 16
;; @004e                               v44 = select_spectre_guard v83, v40, v42  ; v83 = 0
;; @004e                               v45 = load.i64 notrap aligned table v44
;;                                     v84 = iconst.i64 -2
;;                                     v85 = band v45, v84  ; v84 = -2
;; @004e                               brif v45, block5(v85), block4
;;
;;                                 block4 cold:
;;                                     v86 = load.i64 notrap aligned readonly v0+56
;;                                     v87 = load.i64 notrap aligned readonly v86+72
;;                                     v88 = iconst.i32 0
;; @004c                               v36 = iconst.i32 2
;; @004e                               v52 = call_indirect sig0, v87(v0, v88, v36)  ; v88 = 0, v36 = 2
;; @004e                               jump block5(v52)
;;
;;                                 block5(v47: i64):
;; @0050                               brif v47, block13, block12
;;
;;                                 block12 cold:
;; @0050                               trap null_reference
;;
;;                                 block13:
;; @0050                               v53 = load.i64 notrap aligned readonly v47+16
;; @0050                               v54 = load.i64 notrap aligned readonly v47+32
;; @0050                               v55 = call_indirect sig1, v53(v54, v0, v2, v3, v4, v5)
;; @0059                               jump block1
;;
;;                                 block1:
;; @0054                               v57 = iadd.i32 v55, v29
;;                                     v6 -> v57
;; @0059                               return v57
;; }
