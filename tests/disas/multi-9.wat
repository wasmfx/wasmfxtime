;;! target = "x86_64"

(module
  (func (export "f") (param i64 i32) (result i64)
    (local.get 0)
    (local.get 1)
    (local.get 1)
    ;; If with else. More params than results.
    (if (param i64 i32) (result i64)
      (then
        (drop)
        (drop)
        (i64.const -1))
      (else
        (drop)
        (drop)
        (i64.const -2)))))

;; function u0:0(i64 vmctx, i64, i64, i32) -> i64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i64, v3: i32):
;; @0020                               v5 = global_value.i64 gv3
;; @0020                               v6 = load.i64 notrap aligned v5+8
;; @0027                               brif v3, block2, block4(v2, v3)
;;
;;                                 block2:
;; @002b                               v10 = iconst.i64 -1
;; @002d                               jump block3(v10)  ; v10 = -1
;;
;;                                 block4(v8: i64, v9: i32):
;; @0030                               v11 = iconst.i64 -2
;; @0032                               jump block3(v11)  ; v11 = -2
;;
;;                                 block3(v7: i64):
;; @0033                               jump block1(v7)
;;
;;                                 block1(v4: i64):
;; @0033                               return v4
;; }
