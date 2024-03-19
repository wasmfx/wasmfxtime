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
  ;; elements are nullable function references, and whose default
  ;; value is `null`.
  (table $ic-sites 100 100 (ref null $ic-stub))

  (func $ic1 (param i32 i32 i32 i32) (result i32)
        local.get 0)

  ;; A function which uses ICs through `table.get` plus `call_ref`
  (func $call-ics-with-call-ref (param i32 i32 i32 i32) (result i32)
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

        local.get $sum)

  ;; Same as the above function, but uses `call_indirect` rather than
  ;; `call_ref`.
  (func $call-ics-with-call-indirect (param i32 i32 i32 i32) (result i32)
        (local $sum i32)

        ;; IC callsite index 1 (arbitrary).
        local.get 0
        local.get 1
        local.get 2
        local.get 3
        i32.const 1
        call_indirect $ic-sites (type $ic-stub)
        local.get $sum
        i32.add
        local.set $sum

        ;; IC callsite index 2 (arbitrary).
        local.get 0
        local.get 1
        local.get 2
        local.get 3
        i32.const 2
        call_indirect $ic-sites (type $ic-stub)
        local.get $sum
        i32.add
        local.set $sum

        local.get $sum)

  (global $ic-site0 (mut (ref $ic-stub)) (ref.func $ic1))
  (global $ic-site1 (mut (ref $ic-stub)) (ref.func $ic1))

  ;; Sort of similar to the previous two functions, but uses globals instead of
  ;; tables to store ICs. Mostly just here for comparison in terms of codegen.
  (func $call-ics-with-global-get (param i32 i32 i32 i32) (result i32)
        (local $sum i32)

        ;; IC callsite index 1 (arbitrary).
        local.get 0
        local.get 1
        local.get 2
        local.get 3
        global.get $ic-site0
        call_ref $ic-stub
        local.get $sum
        i32.add
        local.set $sum

        ;; IC callsite index 2 (arbitrary).
        local.get 0
        local.get 1
        local.get 2
        local.get 3
        global.get $ic-site1
        call_ref $ic-stub
        local.get $sum
        i32.add
        local.set $sum

        local.get $sum)
)

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
;; @0039                               jump block1
;;
;;                                 block1:
;; @0039                               return v2
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
;; @0048                               v14 = load.i64 notrap aligned v0+72
;;                                     v64 = iconst.i8 0
;; @0048                               v17 = iconst.i64 0
;;                                     v72 = iconst.i64 8
;; @0048                               v16 = iadd v14, v72  ; v72 = 8
;; @0048                               v18 = select_spectre_guard v64, v17, v16  ; v64 = 0, v17 = 0
;; @0048                               v19 = load.i64 table_oob aligned table v18
;;                                     v60 = iconst.i64 -2
;; @0048                               v20 = band v19, v60  ; v60 = -2
;; @0048                               brif v19, block3(v20), block2
;;
;;                                 block2 cold:
;; @005b                               v50 = load.i64 notrap aligned readonly v0+56
;; @005b                               v51 = load.i64 notrap aligned readonly v50+72
;; @003c                               v7 = iconst.i32 0
;;                                     v30 -> v7
;; @0046                               v10 = iconst.i32 1
;; @0048                               v26 = call_indirect sig0, v51(v0, v7, v10)  ; v7 = 0, v10 = 1
;; @0048                               jump block3(v26)
;;
;;                                 block3(v21: i64):
;; @004a                               v27 = load.i64 null_reference aligned readonly v21+16
;; @004a                               v28 = load.i64 notrap aligned readonly v21+32
;; @004a                               v29 = call_indirect sig1, v27(v28, v0, v2, v3, v4, v5)
;; @005b                               v40 = load.i64 notrap aligned v0+72
;;                                     v81 = iconst.i8 0
;;                                     v82 = iconst.i64 0
;;                                     v80 = iconst.i64 16
;; @005b                               v42 = iadd v40, v80  ; v80 = 16
;; @005b                               v44 = select_spectre_guard v81, v82, v42  ; v81 = 0, v82 = 0
;; @005b                               v45 = load.i64 table_oob aligned table v44
;;                                     v83 = iconst.i64 -2
;;                                     v84 = band v45, v83  ; v83 = -2
;; @005b                               brif v45, block5(v84), block4
;;
;;                                 block4 cold:
;;                                     v85 = load.i64 notrap aligned readonly v0+56
;;                                     v86 = load.i64 notrap aligned readonly v85+72
;;                                     v87 = iconst.i32 0
;; @0059                               v36 = iconst.i32 2
;; @005b                               v52 = call_indirect sig0, v86(v0, v87, v36)  ; v87 = 0, v36 = 2
;; @005b                               jump block5(v52)
;;
;;                                 block5(v47: i64):
;; @005d                               v53 = load.i64 null_reference aligned readonly v47+16
;; @005d                               v54 = load.i64 notrap aligned readonly v47+32
;; @005d                               v55 = call_indirect sig1, v53(v54, v0, v2, v3, v4, v5)
;; @0066                               jump block1
;;
;;                                 block1:
;; @0061                               v57 = iadd.i32 v55, v29
;;                                     v6 -> v57
;; @0066                               return v57
;; }
;;
;; function u0:2(i64 vmctx, i64, i32, i32, i32, i32) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+72
;;     sig0 = (i64 vmctx, i64, i32, i32, i32, i32) -> i32 fast
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i64 system_v
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
;; @0075                               v14 = load.i64 notrap aligned v0+72
;;                                     v64 = iconst.i8 0
;; @0075                               v17 = iconst.i64 0
;;                                     v72 = iconst.i64 8
;; @0075                               v16 = iadd v14, v72  ; v72 = 8
;; @0075                               v18 = select_spectre_guard v64, v17, v16  ; v64 = 0, v17 = 0
;; @0075                               v19 = load.i64 table_oob aligned table v18
;;                                     v60 = iconst.i64 -2
;; @0075                               v20 = band v19, v60  ; v60 = -2
;; @0075                               brif v19, block3(v20), block2
;;
;;                                 block2 cold:
;; @0087                               v50 = load.i64 notrap aligned readonly v0+56
;; @0087                               v51 = load.i64 notrap aligned readonly v50+72
;; @0069                               v7 = iconst.i32 0
;;                                     v30 -> v7
;; @0073                               v10 = iconst.i32 1
;; @0075                               v26 = call_indirect sig1, v51(v0, v7, v10)  ; v7 = 0, v10 = 1
;; @0075                               jump block3(v26)
;;
;;                                 block3(v21: i64):
;; @0075                               v27 = load.i64 icall_null aligned readonly v21+16
;; @0075                               v28 = load.i64 notrap aligned readonly v21+32
;; @0075                               v29 = call_indirect sig0, v27(v28, v0, v2, v3, v4, v5)
;; @0087                               v40 = load.i64 notrap aligned v0+72
;;                                     v81 = iconst.i8 0
;;                                     v82 = iconst.i64 0
;;                                     v80 = iconst.i64 16
;; @0087                               v42 = iadd v40, v80  ; v80 = 16
;; @0087                               v44 = select_spectre_guard v81, v82, v42  ; v81 = 0, v82 = 0
;; @0087                               v45 = load.i64 table_oob aligned table v44
;;                                     v83 = iconst.i64 -2
;;                                     v84 = band v45, v83  ; v83 = -2
;; @0087                               brif v45, block5(v84), block4
;;
;;                                 block4 cold:
;;                                     v85 = load.i64 notrap aligned readonly v0+56
;;                                     v86 = load.i64 notrap aligned readonly v85+72
;;                                     v87 = iconst.i32 0
;; @0085                               v36 = iconst.i32 2
;; @0087                               v52 = call_indirect sig1, v86(v0, v87, v36)  ; v87 = 0, v36 = 2
;; @0087                               jump block5(v52)
;;
;;                                 block5(v47: i64):
;; @0087                               v53 = load.i64 icall_null aligned readonly v47+16
;; @0087                               v54 = load.i64 notrap aligned readonly v47+32
;; @0087                               v55 = call_indirect sig0, v53(v54, v0, v2, v3, v4, v5)
;; @0091                               jump block1
;;
;;                                 block1:
;; @008c                               v57 = iadd.i32 v55, v29
;;                                     v6 -> v57
;; @0091                               return v57
;; }
;;
;; function u0:3(i64 vmctx, i64, i32, i32, i32, i32) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i64, i32, i32, i32, i32) -> i32 fast
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32, v3: i32, v4: i32, v5: i32):
;;                                     v8 -> v0
;;                                     v10 -> v0
;;                                     v16 -> v0
;; @009e                               v11 = load.i64 notrap aligned table v0+96
;; @00a0                               v12 = load.i64 null_reference aligned readonly v11+16
;; @00a0                               v13 = load.i64 notrap aligned readonly v11+32
;; @00a0                               v14 = call_indirect sig0, v12(v13, v0, v2, v3, v4, v5)
;; @00af                               v17 = load.i64 notrap aligned table v0+112
;; @00b1                               v18 = load.i64 null_reference aligned readonly v17+16
;; @00b1                               v19 = load.i64 notrap aligned readonly v17+32
;; @00b1                               v20 = call_indirect sig0, v18(v19, v0, v2, v3, v4, v5)
;; @00ba                               jump block1
;;
;;                                 block1:
;; @00b5                               v21 = iadd.i32 v20, v14
;;                                     v6 -> v21
;; @00ba                               return v21
;; }
