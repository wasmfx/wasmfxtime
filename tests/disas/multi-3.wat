;;! target = "x86_64"

(module
  (func (export "multiIf") (param i32 i64 i64) (result i64 i64)
    (local.get 2)
    (local.get 1)
    (local.get 0)
    (if (param i64 i64) (result i64 i64)
      (then return)
      ;; Hits the code path for an `else` after a block that ends unreachable.
      (else
        (drop)
        (drop)
        (i64.const 0)
        (i64.const 0)))))

;; function u0:0(i64 vmctx, i64, i32, i64, i64) -> i64, i64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32, v3: i64, v4: i64):
;; @002f                               v7 = global_value.i64 gv3
;; @002f                               v8 = load.i64 notrap aligned v7+8
;; @0036                               brif v2, block2, block4(v4, v3)
;;
;;                                 block2:
;; @0038                               return v4, v3
;;
;;                                 block4(v11: i64, v12: i64):
;; @003c                               v13 = iconst.i64 0
;; @003e                               v14 = iconst.i64 0
;; @0040                               jump block3(v13, v14)  ; v13 = 0, v14 = 0
;;
;;                                 block3(v9: i64, v10: i64):
;; @0041                               jump block1(v9, v10)
;;
;;                                 block1(v5: i64, v6: i64):
;; @0041                               return v5, v6
;; }
