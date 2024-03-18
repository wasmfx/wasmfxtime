;;! target = "x86_64"

(module
  (func (export "multiIf2") (param i32 i64 i64) (result i64 i64)
    (local.get 2)
    (local.get 1)
    (local.get 0)
    (if (param i64 i64) (result i64 i64)
      (then
        i64.add
        i64.const 1)
      ;; Hits the code path for an `else` after a block that does not end unreachable.
      (else
        i64.sub
        i64.const 2))))

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
;; @0030                               v7 = global_value.i64 gv3
;; @0030                               v8 = load.i64 notrap aligned v7+8
;; @0037                               brif v2, block2, block4(v4, v3)
;;
;;                                 block2:
;; @0039                               v11 = iadd.i64 v4, v3
;; @003a                               v12 = iconst.i64 1
;; @003c                               jump block3(v11, v12)  ; v12 = 1
;;
;;                                 block4(v13: i64, v14: i64):
;; @003d                               v15 = isub.i64 v4, v3
;; @003e                               v16 = iconst.i64 2
;; @0040                               jump block3(v15, v16)  ; v16 = 2
;;
;;                                 block3(v9: i64, v10: i64):
;; @0041                               jump block1(v9, v10)
;;
;;                                 block1(v5: i64, v6: i64):
;; @0041                               return v5, v6
;; }
