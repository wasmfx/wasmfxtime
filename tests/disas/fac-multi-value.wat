;;! target = "x86_64"

(module
  ;; Iterative factorial without locals.
  (func $pick0 (param i64) (result i64 i64)
    (local.get 0) (local.get 0)
  )
  (func $pick1 (param i64 i64) (result i64 i64 i64)
    (local.get 0) (local.get 1) (local.get 0)
  )
  (func (export "fac-ssa") (param i64) (result i64)
    (i64.const 1) (local.get 0)
    (loop $l (param i64 i64) (result i64)
      (call $pick1) (call $pick1) (i64.mul)
      (call $pick1) (i64.const 1) (i64.sub)
      (call $pick0) (i64.const 0) (i64.gt_u)
      (br_if $l)
      (drop) (return)
    )
  )
)

;; function u0:0(i64 vmctx, i64, i64) -> i64, i64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i64):
;; @003b                               v5 = global_value.i64 gv3
;; @003b                               v6 = load.i64 notrap aligned v5+8
;; @0040                               jump block1(v2, v2)
;;
;;                                 block1(v3: i64, v4: i64):
;; @0040                               return v3, v4
;; }
;;
;; function u0:1(i64 vmctx, i64, i64, i64) -> i64, i64, i64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i64, v3: i64):
;; @0042                               v7 = global_value.i64 gv3
;; @0042                               v8 = load.i64 notrap aligned v7+8
;; @0049                               jump block1(v2, v3, v2)
;;
;;                                 block1(v4: i64, v5: i64, v6: i64):
;; @0049                               return v4, v5, v6
;; }
;;
;; function u0:2(i64 vmctx, i64, i64) -> i64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i64, i64, i64) -> i64, i64, i64 fast
;;     sig1 = (i64 vmctx, i64, i64) -> i64, i64 fast
;;     sig2 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig3 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     fn0 = colocated u0:1 sig0
;;     fn1 = colocated u0:0 sig1
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i64):
;; @004b                               v4 = global_value.i64 gv3
;; @004b                               v5 = load.i64 notrap aligned v4+8
;; @004c                               v6 = iconst.i64 1
;; @0050                               jump block2(v6, v2)  ; v6 = 1
;;
;;                                 block2(v7: i64, v8: i64):
;; @0052                               v10, v11, v12 = call fn0(v0, v0, v7, v8)
;; @0054                               v13, v14, v15 = call fn0(v0, v0, v11, v12)
;; @0056                               v16 = imul v14, v15
;; @0057                               v17, v18, v19 = call fn0(v0, v0, v13, v16)
;; @0059                               v20 = iconst.i64 1
;; @005b                               v21 = isub v19, v20  ; v20 = 1
;; @005c                               v22, v23 = call fn1(v0, v0, v21)
;; @005e                               v24 = iconst.i64 0
;; @0060                               v25 = icmp ugt v23, v24  ; v24 = 0
;; @0060                               v26 = uextend.i32 v25
;; @0061                               brif v26, block2(v18, v22), block4
;;
;;                                 block4:
;; @0064                               return v18
;; }
