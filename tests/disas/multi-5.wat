;;! target = "x86_64"

(module
  (func (export "foo")
    i32.const 1
    i64.const 2
    ;; More params than results.
    (block (param i32 i64) (result i32)
      drop
    )
    drop
  )
)

;; function u0:0(i64 vmctx, i64) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @0025                               v2 = global_value.i64 gv3
;; @0025                               v3 = load.i64 notrap aligned v2+8
;; @0026                               v4 = iconst.i32 1
;; @0028                               v5 = iconst.i64 2
;; @002d                               jump block2(v4)  ; v4 = 1
;;
;;                                 block2(v6: i32):
;; @002f                               jump block1
;;
;;                                 block1:
;; @002f                               return
;; }
